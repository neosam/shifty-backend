use std::sync::Arc;

use dao::{
    vacation_entitlement_offset::{
        MockVacationEntitlementOffsetDao, VacationEntitlementOffsetEntity,
    },
    MockTransaction, MockTransactionDao,
};
use mockall::predicate::{always, eq};
use service::{
    clock::MockClockService,
    permission::Authentication,
    uuid_service::MockUuidService,
    vacation_entitlement_offset::{VacationEntitlementOffset, VacationEntitlementOffsetService},
    MockPermissionService, ServiceError,
};
use uuid::{uuid, Uuid};

use crate::test::error_test::test_forbidden;
use crate::vacation_entitlement_offset::{
    VacationEntitlementOffsetServiceDeps, VacationEntitlementOffsetServiceImpl,
};

pub struct VacationEntitlementOffsetServiceDependencies {
    pub vacation_entitlement_offset_dao: MockVacationEntitlementOffsetDao,
    pub permission_service: MockPermissionService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
    pub transaction_dao: MockTransactionDao,
}

impl VacationEntitlementOffsetServiceDeps for VacationEntitlementOffsetServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type VacationEntitlementOffsetDao = MockVacationEntitlementOffsetDao;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = MockTransactionDao;
}

impl VacationEntitlementOffsetServiceDependencies {
    pub fn build_service(
        self,
    ) -> VacationEntitlementOffsetServiceImpl<VacationEntitlementOffsetServiceDependencies> {
        VacationEntitlementOffsetServiceImpl {
            vacation_entitlement_offset_dao: Arc::new(self.vacation_entitlement_offset_dao),
            permission_service: Arc::new(self.permission_service),
            clock_service: Arc::new(self.clock_service),
            uuid_service: Arc::new(self.uuid_service),
            transaction_dao: Arc::new(self.transaction_dao),
        }
    }
}

pub fn build_dependencies(
    permission: bool,
    role: &'static str,
) -> VacationEntitlementOffsetServiceDependencies {
    let vacation_entitlement_offset_dao = MockVacationEntitlementOffsetDao::new();

    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(always(), always())
        .returning(move |inner_role, context| {
            if context == Authentication::Full || (permission && inner_role == role) {
                Ok(())
            } else {
                Err(service::ServiceError::Forbidden)
            }
        });

    let mut clock_service = MockClockService::new();
    clock_service.expect_date_time_now().returning(|| {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
            time::Time::from_hms(23, 42, 0).unwrap(),
        )
    });

    let uuid_service = MockUuidService::new();

    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    VacationEntitlementOffsetServiceDependencies {
        vacation_entitlement_offset_dao,
        permission_service,
        clock_service,
        uuid_service,
        transaction_dao,
    }
}

pub fn default_id() -> Uuid {
    uuid!("67D91F86-2EC7-4FA6-8EB4-9C76A2D4C6E0")
}
pub fn default_version() -> Uuid {
    uuid!("CCB5F4E2-8C7D-4388-AC4E-641D43ADF580")
}
pub fn alternate_version() -> Uuid {
    uuid!("CCB5F4E2-8C7D-4388-AC4E-641D43ADF581")
}
pub fn default_sales_person_id() -> Uuid {
    uuid!("e3ecccf2-356f-408a-ab6c-cd668bd27f80")
}
pub fn fixed_datetime() -> time::PrimitiveDateTime {
    time::PrimitiveDateTime::new(
        time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
        time::Time::from_hms(23, 42, 0).unwrap(),
    )
}

pub fn existing_offset_entity(offset_days: i32) -> VacationEntitlementOffsetEntity {
    VacationEntitlementOffsetEntity {
        id: default_id(),
        sales_person_id: default_sales_person_id(),
        year: 2063,
        offset_days,
        created: fixed_datetime(),
        deleted: None,
        version: default_version(),
    }
}

/// set as HR with no existing row → a new offset is created and the
/// returned domain carries the offset_days. (VAC-OFFSET-01)
#[tokio::test]
pub async fn test_set_hr_creates_offset() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .vacation_entitlement_offset_dao
        .expect_find_by_sales_person_id_and_year()
        .with(eq(default_sales_person_id()), eq(2063), always())
        .returning(|_, _, _| Ok(None));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("vacation-entitlement-offset-service::create id"))
        .returning(|_| default_id());
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("vacation-entitlement-offset-service::create version"))
        .returning(|_| default_version());
    dependencies
        .vacation_entitlement_offset_dao
        .expect_create()
        .with(
            eq(existing_offset_entity(3)),
            eq("vacation-entitlement-offset-service::create"),
            always(),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));

    let service = dependencies.build_service();
    let result = service
        .set(default_sales_person_id(), 2063, 3, ().into(), None)
        .await
        .unwrap();

    assert_eq!(result.offset_days, 3);
    assert_eq!(result.id, default_id());
    assert_eq!(result.version, default_version());
}

/// get as HR returns Some(offset_days). (VAC-OFFSET-01 / D-28-03 read gate)
#[tokio::test]
pub async fn test_get_hr_returns_offset() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .vacation_entitlement_offset_dao
        .expect_find_by_sales_person_id_and_year()
        .with(eq(default_sales_person_id()), eq(2063), always())
        .returning(|_, _, _| Ok(Some(existing_offset_entity(5))));

    let service = dependencies.build_service();
    let result: Option<VacationEntitlementOffset> = service
        .get(default_sales_person_id(), 2063, ().into(), None)
        .await
        .unwrap();

    assert_eq!(result.map(|o| o.offset_days), Some(5));
}

/// A second set updates the SAME active (person, year) row instead of
/// creating a duplicate: find returns an existing row → update is called,
/// create is not. (VAC-OFFSET-01 single-active-row)
#[tokio::test]
pub async fn test_set_hr_updates_existing_row() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .vacation_entitlement_offset_dao
        .expect_find_by_sales_person_id_and_year()
        .with(eq(default_sales_person_id()), eq(2063), always())
        .returning(|_, _, _| Ok(Some(existing_offset_entity(3))));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("vacation-entitlement-offset-service::update version"))
        .returning(|_| alternate_version());
    // No create on the update path.
    dependencies
        .vacation_entitlement_offset_dao
        .expect_create()
        .times(0)
        .returning(|_, _, _| Ok(()));
    dependencies
        .vacation_entitlement_offset_dao
        .expect_update()
        .with(
            eq(VacationEntitlementOffsetEntity {
                offset_days: -2,
                version: alternate_version(),
                ..existing_offset_entity(3)
            }),
            eq("vacation-entitlement-offset-service::update"),
            always(),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));

    let service = dependencies.build_service();
    let result = service
        .set(default_sales_person_id(), 2063, -2, ().into(), None)
        .await
        .unwrap();

    assert_eq!(result.offset_days, -2);
    assert_eq!(result.id, default_id());
    assert_eq!(result.version, alternate_version());
}

/// delete as HR soft-deletes the active row (sets deleted + new version).
#[tokio::test]
pub async fn test_delete_hr_soft_deletes() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .vacation_entitlement_offset_dao
        .expect_find_by_sales_person_id_and_year()
        .with(eq(default_sales_person_id()), eq(2063), always())
        .returning(|_, _, _| Ok(Some(existing_offset_entity(3))));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("vacation-entitlement-offset-service::delete version"))
        .returning(|_| alternate_version());
    dependencies
        .vacation_entitlement_offset_dao
        .expect_update()
        .with(
            eq(VacationEntitlementOffsetEntity {
                deleted: Some(fixed_datetime()),
                version: alternate_version(),
                ..existing_offset_entity(3)
            }),
            eq("vacation-entitlement-offset-service::delete"),
            always(),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));

    let service = dependencies.build_service();
    service
        .delete(default_sales_person_id(), 2063, ().into(), None)
        .await
        .unwrap();
}

/// delete as HR when no active row exists → EntityNotFoundGeneric.
#[tokio::test]
pub async fn test_delete_hr_not_found() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .vacation_entitlement_offset_dao
        .expect_find_by_sales_person_id_and_year()
        .with(eq(default_sales_person_id()), eq(2063), always())
        .returning(|_, _, _| Ok(None));

    let service = dependencies.build_service();
    let result = service
        .delete(default_sales_person_id(), 2063, ().into(), None)
        .await;

    assert!(matches!(
        result,
        Err(ServiceError::EntityNotFoundGeneric(_))
    ));
}

/// HR-gate: a non-HR caller's `set` returns Forbidden and NO DAO write
/// (create OR update) ever happens. (D-28-06b / VAC-OFFSET-01 offset_hr_gate)
#[tokio::test]
pub async fn test_set_non_hr_forbidden_no_write() {
    let mut dependencies = build_dependencies(false, "hr");
    // Prove zero writes on the denied path.
    dependencies
        .vacation_entitlement_offset_dao
        .expect_create()
        .times(0)
        .returning(|_, _, _| Ok(()));
    dependencies
        .vacation_entitlement_offset_dao
        .expect_update()
        .times(0)
        .returning(|_, _, _| Ok(()));

    let service = dependencies.build_service();
    let result = service
        .set(default_sales_person_id(), 2063, 4, ().into(), None)
        .await;

    test_forbidden(&result);
}

/// HR-gate also applies to the read path: non-HR `get` → Forbidden.
#[tokio::test]
pub async fn test_get_non_hr_forbidden() {
    let dependencies = build_dependencies(false, "hr");
    let service = dependencies.build_service();
    let result = service
        .get(default_sales_person_id(), 2063, ().into(), None)
        .await;

    test_forbidden(&result);
}
