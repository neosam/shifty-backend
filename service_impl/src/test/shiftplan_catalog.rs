use crate::test::error_test::*;
use dao::{
    shiftplan::{MockShiftplanDao, ShiftplanEntity},
    MockTransaction, MockTransactionDao,
};
use service::{
    clock::MockClockService,
    permission::MockPermissionService,
    shiftplan_catalog::{Shiftplan, ShiftplanService},
    uuid_service::MockUuidService,
};
use std::sync::Arc;
use uuid::{uuid, Uuid};

use crate::shiftplan_catalog::{ShiftplanServiceDeps, ShiftplanServiceImpl};

fn default_shiftplan_id() -> Uuid {
    uuid!("00000000-0000-4000-8000-000000000001")
}

fn default_version() -> Uuid {
    uuid!("AAAAAAAA-BBBB-4CCC-8DDD-EEEEEEEEEEEE")
}

fn default_shiftplan_entity() -> ShiftplanEntity {
    ShiftplanEntity {
        id: default_shiftplan_id(),
        name: "main".into(),
        is_planning: false,
        deleted: None,
        version: default_version(),
    }
}

pub struct ShiftplanCatalogServiceDependencies {
    pub shiftplan_dao: MockShiftplanDao,
    pub permission_service: MockPermissionService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
    pub transaction_dao: MockTransactionDao,
}

impl ShiftplanServiceDeps for ShiftplanCatalogServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type ShiftplanDao = MockShiftplanDao;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = MockTransactionDao;
}

impl ShiftplanCatalogServiceDependencies {
    pub fn build_service(
        self,
    ) -> ShiftplanServiceImpl<ShiftplanCatalogServiceDependencies> {
        ShiftplanServiceImpl {
            shiftplan_dao: self.shiftplan_dao.into(),
            permission_service: self.permission_service.into(),
            clock_service: self.clock_service.into(),
            uuid_service: self.uuid_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

fn build_dependencies() -> ShiftplanCatalogServiceDependencies {
    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let clock_service = MockClockService::new();

    let mut uuid_service = MockUuidService::new();
    uuid_service
        .expect_new_uuid()
        .returning(|_| Uuid::new_v4());

    ShiftplanCatalogServiceDependencies {
        shiftplan_dao: MockShiftplanDao::new(),
        permission_service,
        clock_service,
        uuid_service,
        transaction_dao,
    }
}

#[tokio::test]
async fn test_get_all() {
    let mut deps = build_dependencies();
    deps.shiftplan_dao
        .expect_all()
        .returning(|_| Ok(Arc::new([default_shiftplan_entity()])));

    let service = deps.build_service();
    let result = service.get_all(().auth(), None).await;
    assert!(result.is_ok());
    let shiftplans = result.unwrap();
    assert_eq!(shiftplans.len(), 1);
    assert_eq!(shiftplans[0].name.as_ref(), "main");
    assert!(!shiftplans[0].is_planning);
}

#[tokio::test]
async fn test_get_by_id() {
    let mut deps = build_dependencies();
    deps.shiftplan_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(Some(default_shiftplan_entity())));

    let service = deps.build_service();
    let result = service
        .get_by_id(default_shiftplan_id(), ().auth(), None)
        .await;
    assert!(result.is_ok());
    let shiftplan = result.unwrap();
    assert_eq!(shiftplan.name.as_ref(), "main");
}

#[tokio::test]
async fn test_get_by_id_not_found() {
    let mut deps = build_dependencies();
    deps.shiftplan_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(None));

    let service = deps.build_service();
    let id = Uuid::new_v4();
    let result = service.get_by_id(id, ().auth(), None).await;
    test_not_found(&result, &id);
}

#[tokio::test]
async fn test_create() {
    let mut deps = build_dependencies();
    deps.shiftplan_dao
        .expect_create()
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let shiftplan = Shiftplan {
        id: Uuid::nil(),
        name: "Backplan".into(),
        is_planning: true,
        deleted: None,
        version: Uuid::nil(),
    };
    let result = service.create(&shiftplan, ().auth(), None).await;
    assert!(result.is_ok());
    let created = result.unwrap();
    assert_ne!(created.id, Uuid::nil());
    assert_ne!(created.version, Uuid::nil());
    assert_eq!(created.name.as_ref(), "Backplan");
    assert!(created.is_planning);
}

#[tokio::test]
async fn test_create_with_id_fails() {
    let deps = build_dependencies();
    let service = deps.build_service();
    let shiftplan = Shiftplan {
        id: Uuid::new_v4(),
        name: "test".into(),
        is_planning: false,
        deleted: None,
        version: Uuid::nil(),
    };
    let result = service.create(&shiftplan, ().auth(), None).await;
    test_zero_id_error(&result);
}

#[tokio::test]
async fn test_create_with_version_fails() {
    let deps = build_dependencies();
    let service = deps.build_service();
    let shiftplan = Shiftplan {
        id: Uuid::nil(),
        name: "test".into(),
        is_planning: false,
        deleted: None,
        version: Uuid::new_v4(),
    };
    let result = service.create(&shiftplan, ().auth(), None).await;
    test_zero_version_error(&result);
}

#[tokio::test]
async fn test_update() {
    let mut deps = build_dependencies();
    deps.shiftplan_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(Some(default_shiftplan_entity())));
    deps.shiftplan_dao
        .expect_update()
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let shiftplan = Shiftplan {
        id: default_shiftplan_id(),
        name: "Renamed".into(),
        is_planning: false,
        deleted: None,
        version: default_version(),
    };
    let result = service.update(&shiftplan, ().auth(), None).await;
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.name.as_ref(), "Renamed");
    assert_ne!(updated.version, default_version());
}

#[tokio::test]
async fn test_update_version_conflict() {
    let mut deps = build_dependencies();
    deps.shiftplan_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(Some(default_shiftplan_entity())));

    let service = deps.build_service();
    let wrong_version = Uuid::new_v4();
    let shiftplan = Shiftplan {
        id: default_shiftplan_id(),
        name: "Renamed".into(),
        is_planning: false,
        deleted: None,
        version: wrong_version,
    };
    let result = service.update(&shiftplan, ().auth(), None).await;
    test_conflicts(&result, &default_shiftplan_id(), &default_version(), &wrong_version);
}

#[tokio::test]
async fn test_delete() {
    let mut deps = build_dependencies();
    deps.shiftplan_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(Some(default_shiftplan_entity())));
    deps.shiftplan_dao
        .expect_update()
        .returning(|_, _, _| Ok(()));
    deps.clock_service
        .expect_date_time_now()
        .returning(generate_default_datetime);

    let service = deps.build_service();
    let result = service
        .delete(default_shiftplan_id(), ().auth(), None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_not_found() {
    let mut deps = build_dependencies();
    deps.shiftplan_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(None));

    let service = deps.build_service();
    let id = Uuid::new_v4();
    let result = service.delete(id, ().auth(), None).await;
    test_not_found(&result, &id);
}

#[tokio::test]
async fn test_create_forbidden() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Err(service::ServiceError::Forbidden));

    let service = deps.build_service();
    let shiftplan = Shiftplan {
        id: Uuid::nil(),
        name: "test".into(),
        is_planning: false,
        deleted: None,
        version: Uuid::nil(),
    };
    let result = service.create(&shiftplan, ().auth(), None).await;
    test_forbidden(&result);
}
