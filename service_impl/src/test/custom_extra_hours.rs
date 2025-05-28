use std::sync::Arc;

use dao::custom_extra_hours::CustomExtraHoursEntity;
use dao::custom_extra_hours::MockCustomExtraHoursDao;
use dao::MockTransaction;
use dao::MockTransactionDao;
use mockall::predicate::always;
use mockall::predicate::eq;
use service::clock::MockClockService;
use service::custom_extra_hours::CustomExtraHours;
use service::custom_extra_hours::CustomExtraHoursService;
use service::permission::HR_PRIVILEGE;
use service::sales_person::MockSalesPersonService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use time::macros::datetime;
use uuid::uuid;
use uuid::Uuid;

use crate::custom_extra_hours::CustomExtraHoursDeps;
use crate::custom_extra_hours::CustomExtraHoursServiceImpl;
use crate::test::error_test::test_conflicts;
use crate::test::error_test::test_forbidden;
use crate::test::error_test::test_not_found;
use crate::test::error_test::test_zero_id_error;
use crate::test::error_test::test_zero_version_error;

pub fn default_id() -> Uuid {
    uuid!("CEA260A0-112B-4970-936C-F7E529955BD0")
}
pub fn alternate_id() -> Uuid {
    uuid!("CEA260A0-112B-4970-936C-F7E529955BD1")
}
pub fn default_version() -> Uuid {
    uuid!("F79C462A-8D4E-42E1-8171-DB4DBD019E50")
}
pub fn alternate_version() -> Uuid {
    uuid!("F79C462A-8D4E-42E1-8171-DB4DBD019E51")
}
pub fn default_sales_person_id() -> Uuid {
    uuid!("04215DFE-13C4-413C-8C66-77AC741BB5F0")
}

pub fn default_custom_extra_hours() -> CustomExtraHours {
    CustomExtraHours {
        id: default_id(),
        name: "Test Custom Extra Hours".into(),
        description: Some("Test Description".into()),
        modifies_balance: true,
        assigned_sales_person_ids: Arc::from([default_sales_person_id()]),
        created: None,
        deleted: None,
        version: default_version(),
    }
}
pub fn alternative_custom_extra_hours() -> CustomExtraHours {
    CustomExtraHours {
        id: alternate_id(),
        name: "Alternative Custom Extra Hours".into(),
        description: Some("Alternative Test Description".into()),
        modifies_balance: false,
        assigned_sales_person_ids: Arc::from([default_sales_person_id()]),
        created: None,
        deleted: None,
        version: default_version(),
    }
}

pub fn default_custom_extra_hours_entity() -> CustomExtraHoursEntity {
    CustomExtraHoursEntity {
        id: default_id(),
        name: "Test Custom Extra Hours".into(),
        description: Some("Test Description".into()),
        modifies_balance: true,
        assigned_sales_person_ids: Arc::from([default_sales_person_id()]),
        created: time::macros::datetime!(2023-10-01 12:00:00),
        deleted: None,
        version: default_version(),
    }
}

pub fn alternative_custom_extra_hours_entity() -> CustomExtraHoursEntity {
    CustomExtraHoursEntity {
        id: alternate_id(),
        name: "Alternative Custom Extra Hours".into(),
        description: Some("Alternative Test Description".into()),
        modifies_balance: false,
        assigned_sales_person_ids: Arc::from([default_sales_person_id()]),
        created: time::macros::datetime!(2023-10-02 12:00:00),
        deleted: None,
        version: default_version(),
    }
}

struct CustomExtraHoursDependencies {
    custom_extra_hours_dao: MockCustomExtraHoursDao,
    sales_person_service: MockSalesPersonService,
    uuid_service: MockUuidService,
    clock_service: MockClockService,
    permission_service: MockPermissionService,
    transaction_dao: MockTransactionDao,
}

impl CustomExtraHoursDeps for CustomExtraHoursDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type CustomExtraHoursDao = MockCustomExtraHoursDao;
    type SalesPersonService = MockSalesPersonService;
    type UuidService = MockUuidService;
    type ClockService = MockClockService;
    type PermissionService = MockPermissionService;
    type TransactionDao = MockTransactionDao;
}
impl CustomExtraHoursDependencies {
    fn build_service(self) -> CustomExtraHoursServiceImpl<CustomExtraHoursDependencies> {
        CustomExtraHoursServiceImpl {
            custom_extra_hours_dao: self.custom_extra_hours_dao.into(),
            sales_person_service: self.sales_person_service.into(),
            uuid_service: self.uuid_service.into(),
            clock_service: self.clock_service.into(),
            permission_service: self.permission_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

fn build_dependencies() -> CustomExtraHoursDependencies {
    let mut custom_extra_hours_dao = MockCustomExtraHoursDao::new();
    let mut sales_person_service = MockSalesPersonService::new();
    let uuid_service = MockUuidService::new();
    let mut clock_service = MockClockService::new();
    let mut permission_service = MockPermissionService::new();
    let mut transaction_dao = MockTransactionDao::new();

    custom_extra_hours_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(default_custom_extra_hours_entity().into()));
    sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));

    clock_service
        .expect_date_time_now()
        .returning(|| time::macros::datetime!(2023-10-01 12:00:00));

    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    CustomExtraHoursDependencies {
        custom_extra_hours_dao,
        sales_person_service,
        uuid_service,
        clock_service,
        permission_service,
        transaction_dao,
    }
}

#[tokio::test]
async fn test_get_all() {
    let mut deps = build_dependencies();
    deps.custom_extra_hours_dao
        .expect_find_all()
        .returning(|_| {
            Ok([
                default_custom_extra_hours_entity(),
                alternative_custom_extra_hours_entity(),
            ]
            .into())
        });
    let service = deps.build_service();

    let result = service.get_all(().into(), None).await;

    assert!(result.is_ok());
    let mut custom_extra_hours_list: Vec<CustomExtraHours> =
        result.unwrap().iter().cloned().collect();
    custom_extra_hours_list.sort_by(|a, b| a.id.cmp(&b.id));
    assert_eq!(custom_extra_hours_list.len(), 2);
    assert_eq!(
        custom_extra_hours_list[0],
        CustomExtraHours {
            created: Some(datetime!(2023-10-01 12:00:00)),
            ..default_custom_extra_hours()
        }
    );
    assert_eq!(
        custom_extra_hours_list[1],
        CustomExtraHours {
            created: Some(datetime!(2023-10-02 12:00:00)),
            ..alternative_custom_extra_hours()
        }
    );
}

#[tokio::test]
async fn test_get_all_no_permission() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(service::ServiceError::Forbidden));
    let service = deps.build_service();

    let result = service.get_all(().into(), None).await;

    test_forbidden(&result);
}

#[tokio::test]
async fn test_get_by_id() {
    let mut deps = build_dependencies();
    deps.custom_extra_hours_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(default_custom_extra_hours_entity().into()));
    let service = deps.build_service();

    let result = service.get_by_id(default_id(), ().into(), None).await;

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        CustomExtraHours {
            created: Some(datetime!(2023-10-01 12:00:00)),
            ..default_custom_extra_hours()
        }
    );
}

#[tokio::test]
async fn test_get_by_id_not_found() {
    let mut deps = build_dependencies();
    deps.custom_extra_hours_dao.checkpoint();
    deps.custom_extra_hours_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(None));
    let service = deps.build_service();

    let result = service.get_by_id(default_id(), ().into(), None).await;

    assert!(result.is_err());
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_get_by_id_no_permission() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(service::ServiceError::Forbidden));
    let service = deps.build_service();

    let result = service.get_by_id(default_id(), ().into(), None).await;

    test_forbidden(&result);
}

#[tokio::test]
async fn test_get_by_sales_person_id() {
    let mut deps = build_dependencies();
    deps.custom_extra_hours_dao
        .expect_find_by_sales_person_id()
        .returning(|_, _| {
            Ok([
                default_custom_extra_hours_entity(),
                alternative_custom_extra_hours_entity(),
            ]
            .into())
        });
    let service = deps.build_service();

    let result = service
        .get_by_sales_person_id(default_sales_person_id(), ().into(), None)
        .await;

    assert!(result.is_ok());
    let mut custom_extra_hours_list: Vec<CustomExtraHours> =
        result.unwrap().iter().cloned().collect();
    custom_extra_hours_list.sort_by(|a, b| a.id.cmp(&b.id));
    assert_eq!(custom_extra_hours_list.len(), 2);
    assert_eq!(
        custom_extra_hours_list[0],
        CustomExtraHours {
            created: Some(datetime!(2023-10-01 12:00:00)),
            ..default_custom_extra_hours()
        }
    );
    assert_eq!(
        custom_extra_hours_list[1],
        CustomExtraHours {
            created: Some(datetime!(2023-10-02 12:00:00)),
            ..alternative_custom_extra_hours()
        }
    );
}

#[tokio::test]
async fn test_get_by_sales_person_id_no_permission() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(service::ServiceError::Forbidden));
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(service::ServiceError::Forbidden));
    let service = deps.build_service();

    let result = service
        .get_by_sales_person_id(default_sales_person_id(), ().into(), None)
        .await;

    test_forbidden(&result);
}

#[tokio::test]
async fn test_get_by_sales_person_id_sales_person_permission() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(service::ServiceError::Forbidden));
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    deps.custom_extra_hours_dao
        .expect_find_by_sales_person_id()
        .returning(|_, _| Ok([default_custom_extra_hours_entity()].into()));
    let service = deps.build_service();

    let result = service
        .get_by_sales_person_id(default_sales_person_id(), ().into(), None)
        .await;

    assert!(result.is_ok());
    let custom_extra_hours_list: Vec<CustomExtraHours> = result.unwrap().iter().cloned().collect();
    assert_eq!(custom_extra_hours_list.len(), 1);
    assert_eq!(
        custom_extra_hours_list[0],
        CustomExtraHours {
            created: Some(datetime!(2023-10-01 12:00:00)),
            ..default_custom_extra_hours()
        }
    );
}

#[tokio::test]
async fn test_create() {
    let mut deps = build_dependencies();
    deps.custom_extra_hours_dao.checkpoint();
    deps.custom_extra_hours_dao
        .expect_create()
        .returning(|_, _, _| Ok(()));
    deps.custom_extra_hours_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(Some(default_custom_extra_hours_entity().into())));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("create-id"))
        .returning(|_| default_id());
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("create-version"))
        .returning(|_| default_version());
    let service = deps.build_service();

    let result = service
        .create(
            &CustomExtraHours {
                id: Uuid::nil(),
                version: Uuid::nil(),
                ..default_custom_extra_hours()
            },
            ().into(),
            None,
        )
        .await;

    dbg!(&result);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        CustomExtraHours {
            created: Some(datetime!(2023-10-01 12:00:00)),
            ..default_custom_extra_hours()
        }
    );
}

#[tokio::test]
async fn test_create_no_permission() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(service::ServiceError::Forbidden));
    let service = deps.build_service();

    let result = service
        .create(
            &CustomExtraHours {
                id: Uuid::nil(),
                version: Uuid::nil(),
                ..default_custom_extra_hours()
            },
            ().into(),
            None,
        )
        .await;

    test_forbidden(&result);
}

#[tokio::test]
async fn test_create_invalid_id() {
    let mut deps = build_dependencies();
    deps.custom_extra_hours_dao.checkpoint();
    let service = deps.build_service();

    let result = service
        .create(
            &CustomExtraHours {
                version: Uuid::nil(),
                ..default_custom_extra_hours()
            },
            ().into(),
            None,
        )
        .await;

    test_zero_id_error(&result);
}

#[tokio::test]
async fn test_create_invalid_version() {
    let deps = build_dependencies();
    let service = deps.build_service();

    let result = service
        .create(
            &CustomExtraHours {
                id: Uuid::nil(),
                ..default_custom_extra_hours()
            },
            ().into(),
            None,
        )
        .await;

    test_zero_version_error(&result);
}

#[tokio::test]
async fn test_update() {
    let mut deps = build_dependencies();
    deps.custom_extra_hours_dao.checkpoint();
    deps.custom_extra_hours_dao
        .expect_update()
        .returning(|_, _, _| Ok(()));
    deps.custom_extra_hours_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(Some(default_custom_extra_hours_entity().into())));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("update-version"))
        .returning(|_| alternate_version());
    let service = deps.build_service();

    let result = service
        .update(
            &CustomExtraHours {
                created: Some(datetime!(2023-10-01 12:00:00)),
                ..default_custom_extra_hours()
            },
            ().into(),
            None,
        )
        .await;

    assert_eq!(
        result.unwrap(),
        CustomExtraHours {
            created: Some(datetime!(2023-10-01 12:00:00)),
            version: alternate_version(),
            ..default_custom_extra_hours()
        }
    );
}

#[tokio::test]
async fn test_update_no_permission() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(service::ServiceError::Forbidden));
    let service = deps.build_service();

    let result = service
        .update(&default_custom_extra_hours(), ().into(), None)
        .await;

    test_forbidden(&result);
}

#[tokio::test]
async fn test_update_not_found() {
    let mut deps = build_dependencies();
    deps.custom_extra_hours_dao.checkpoint();
    deps.custom_extra_hours_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(None));
    let service = deps.build_service();

    let result = service
        .update(&default_custom_extra_hours(), ().into(), None)
        .await;

    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_update_version_conflict() {
    let mut deps = build_dependencies();
    deps.custom_extra_hours_dao.checkpoint();
    deps.custom_extra_hours_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(Some(default_custom_extra_hours_entity().into())));
    let service = deps.build_service();

    let result = service
        .update(
            &CustomExtraHours {
                version: alternate_version(),
                ..default_custom_extra_hours()
            },
            ().into(),
            None,
        )
        .await;

    test_conflicts(
        &result,
        &default_id(),
        &alternate_version(),
        &default_version(),
    );
}

#[tokio::test]
async fn test_delete() {
    let mut deps = build_dependencies();
    deps.custom_extra_hours_dao.checkpoint();
    deps.custom_extra_hours_dao
        .expect_update()
        .with(
            eq(CustomExtraHoursEntity {
                deleted: Some(time::macros::datetime!(2023-10-01 12:00:00)),
                ..default_custom_extra_hours_entity()
            }),
            always(),
            always(),
        )
        .returning(|_, _, _| Ok(()));
    deps.custom_extra_hours_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(Some(default_custom_extra_hours_entity().into())));
    deps.clock_service
        .expect_date_time_now()
        .returning(|| time::macros::datetime!(2023-10-01 12:00:00));
    let service = deps.build_service();

    let result = service.delete(default_id(), ().into(), None).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_no_permission() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(service::ServiceError::Forbidden));
    let service = deps.build_service();

    let result = service.delete(default_id(), ().into(), None).await;

    test_forbidden(&result);
}

#[tokio::test]
async fn test_delete_not_found() {
    let mut deps = build_dependencies();
    deps.custom_extra_hours_dao.checkpoint();
    deps.custom_extra_hours_dao
        .expect_find_by_id()
        .returning(|_, _| Ok(None));
    let service = deps.build_service();

    let result = service.delete(default_id(), ().into(), None).await;

    test_not_found(&result, &default_id());
}
