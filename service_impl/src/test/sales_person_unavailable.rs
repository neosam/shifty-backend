use std::sync::Arc;

use dao::sales_person_unavailable::{MockSalesPersonUnavailableDao, SalesPersonUnavailableEntity};
use mockall::predicate::{always, eq};
use service::{
    clock::MockClockService,
    permission::Authentication,
    sales_person::MockSalesPersonService,
    sales_person_unavailable::{SalesPersonUnavailable, SalesPersonUnavailableService},
    uuid_service::MockUuidService,
    MockPermissionService, ServiceError,
};

use uuid::{uuid, Uuid};

use crate::sales_person_unavailable::SalesPersonUnavailableServiceImpl;
use crate::test::error_test::{
    test_exists_error, test_forbidden, test_not_found, test_zero_id_error, test_zero_version_error,
};

pub struct SalesPersonUnavailableServiceDependencies {
    pub sales_person_unavailable_dao: MockSalesPersonUnavailableDao,
    pub sales_person_service: MockSalesPersonService,
    pub permission_service: MockPermissionService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
}
impl SalesPersonUnavailableServiceDependencies {
    pub fn build_service(
        self,
    ) -> SalesPersonUnavailableServiceImpl<
        MockSalesPersonUnavailableDao,
        MockSalesPersonService,
        MockPermissionService,
        MockClockService,
        MockUuidService,
    > {
        SalesPersonUnavailableServiceImpl {
            sales_person_unavailable_dao: Arc::new(self.sales_person_unavailable_dao),
            sales_person_service: Arc::new(self.sales_person_service),
            permission_service: Arc::new(self.permission_service),
            clock_service: Arc::new(self.clock_service),
            uuid_service: Arc::new(self.uuid_service),
        }
    }
}

pub fn build_dependencies(
    permission: bool,
    role: &'static str,
) -> SalesPersonUnavailableServiceDependencies {
    let sales_person_unavailable_dao = MockSalesPersonUnavailableDao::new();
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(always(), always())
        .returning(move |inner_role, context| {
            if context == Authentication::Full || (permission && inner_role == role) {
                println!("Permission granted");
                Ok(())
            } else {
                println!("Permission denied");
                Err(service::ServiceError::Forbidden)
            }
        });
    permission_service
        .expect_current_user_id()
        .returning(|_| Ok(Some("TESTUSER".into())));

    let mut clock_service = MockClockService::new();
    clock_service
        .expect_time_now()
        .returning(|| time::Time::from_hms(23, 42, 0).unwrap());
    clock_service
        .expect_date_now()
        .returning(|| time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap());
    clock_service.expect_date_time_now().returning(|| {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
            time::Time::from_hms(23, 42, 0).unwrap(),
        )
    });
    let uuid_service = MockUuidService::new();

    let sales_person_service = MockSalesPersonService::new();

    SalesPersonUnavailableServiceDependencies {
        sales_person_unavailable_dao,
        sales_person_service,
        permission_service,
        clock_service,
        uuid_service,
    }
}

pub fn default_id() -> uuid::Uuid {
    uuid!("67D91F86-2EC7-4FA6-8EB4-9C76A2D4C6E0")
}
pub fn alternate_id() -> uuid::Uuid {
    uuid!("67D91F86-2EC7-4FA6-8EB4-9C76A2D4C6E1")
}

pub fn default_version() -> uuid::Uuid {
    uuid!("CCB5F4E2-8C7D-4388-AC4E-641D43ADF580")
}
pub fn alternate_version() -> uuid::Uuid {
    uuid!("CCB5F4E2-8C7D-4388-AC4E-641D43ADF581")
}

pub fn default_sales_person_id() -> uuid::Uuid {
    uuid!("e3ecccf2-356f-408a-ab6c-cd668bd27f80")
}

pub fn default_sales_person_unavailable_entity() -> SalesPersonUnavailableEntity {
    SalesPersonUnavailableEntity {
        id: default_id(),
        sales_person_id: default_sales_person_id(),
        year: 2063,
        calendar_week: 42,
        day_of_week: dao::slot::DayOfWeek::Friday,
        created: time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
            time::Time::from_hms(23, 42, 0).unwrap(),
        ),
        deleted: None,
        version: default_version(),
    }
}

pub fn alternate_sales_person_unavailable_entity() -> SalesPersonUnavailableEntity {
    SalesPersonUnavailableEntity {
        id: alternate_id(),
        sales_person_id: default_sales_person_id(),
        year: 2063,
        calendar_week: 42,
        day_of_week: dao::slot::DayOfWeek::Friday,
        created: time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
            time::Time::from_hms(23, 42, 0).unwrap(),
        ),
        deleted: None,
        version: default_version(),
    }
}

pub fn default_sales_person_unavailable(
) -> service::sales_person_unavailable::SalesPersonUnavailable {
    service::sales_person_unavailable::SalesPersonUnavailable {
        id: default_id(),
        sales_person_id: default_sales_person_id(),
        year: 2063,
        calendar_week: 42,
        day_of_week: service::slot::DayOfWeek::Friday,
        created: Some(time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
            time::Time::from_hms(23, 42, 0).unwrap(),
        )),
        deleted: None,
        version: default_version(),
    }
}

pub fn alternate_sales_person_unavailable(
) -> service::sales_person_unavailable::SalesPersonUnavailable {
    service::sales_person_unavailable::SalesPersonUnavailable {
        id: alternate_id(),
        sales_person_id: default_sales_person_id(),
        year: 2063,
        calendar_week: 42,
        day_of_week: service::slot::DayOfWeek::Friday,
        created: Some(time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
            time::Time::from_hms(23, 42, 0).unwrap(),
        )),
        deleted: None,
        version: default_version(),
    }
}

#[tokio::test]
pub async fn test_get_all_shiftplanner() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies
        .sales_person_unavailable_dao
        .expect_find_all_by_sales_person_id()
        .with(eq(default_sales_person_id()))
        .returning(|_| {
            Ok([
                default_sales_person_unavailable_entity(),
                alternate_sales_person_unavailable_entity(),
            ]
            .into())
        });
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Err(ServiceError::Forbidden));
    let service = dependencies.build_service();

    let result = service
        .get_all_for_sales_person(default_sales_person_id(), ().into())
        .await
        .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], default_sales_person_unavailable());
    assert_eq!(result[1], alternate_sales_person_unavailable());
}

#[tokio::test]
pub async fn test_get_all_sales_person() {
    let mut dependencies = build_dependencies(false, "shiftplanner");
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Ok(()));
    dependencies
        .sales_person_unavailable_dao
        .expect_find_all_by_sales_person_id()
        .with(eq(default_sales_person_id()))
        .returning(|_| Ok([default_sales_person_unavailable_entity()].into()));
    let service = dependencies.build_service();

    let result = service
        .get_all_for_sales_person(default_sales_person_id(), ().into())
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], default_sales_person_unavailable());
}

#[tokio::test]
pub async fn test_get_all_no_permission() {
    let mut dependencies = build_dependencies(false, "sales");
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Err(ServiceError::Forbidden));
    let service = dependencies.build_service();

    let result = service
        .get_all_for_sales_person(default_sales_person_id(), ().into())
        .await;
    test_forbidden(&result);
}

#[tokio::test]
pub async fn test_get_by_week_shiftplanner() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_week_and_sales_person_id()
        .with(eq(default_sales_person_id()), eq(2063), eq(42))
        .returning(|_, _, _| {
            Ok([
                default_sales_person_unavailable_entity(),
                alternate_sales_person_unavailable_entity(),
            ]
            .into())
        });
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Err(ServiceError::Forbidden));
    let service = dependencies.build_service();

    let result = service
        .get_by_week_for_sales_person(default_sales_person_id(), 2063, 42, ().into())
        .await
        .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], default_sales_person_unavailable());
    assert_eq!(result[1], alternate_sales_person_unavailable());
}

#[tokio::test]
pub async fn test_get_by_week_sales_person() {
    let mut dependencies = build_dependencies(false, "sales");
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_week_and_sales_person_id()
        .with(eq(default_sales_person_id()), eq(2063), eq(42))
        .returning(|_, _, _| {
            Ok([
                default_sales_person_unavailable_entity(),
                alternate_sales_person_unavailable_entity(),
            ]
            .into())
        });
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Ok(()));
    let service = dependencies.build_service();

    let result = service
        .get_by_week_for_sales_person(default_sales_person_id(), 2063, 42, ().into())
        .await
        .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], default_sales_person_unavailable());
    assert_eq!(result[1], alternate_sales_person_unavailable());
}

#[tokio::test]
pub async fn test_get_by_week_no_permission() {
    let mut dependencies = build_dependencies(false, "sales");
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_week_and_sales_person_id()
        .with(eq(default_sales_person_id()), eq(2063), eq(42))
        .returning(|_, _, _| {
            Ok([
                default_sales_person_unavailable_entity(),
                alternate_sales_person_unavailable_entity(),
            ]
            .into())
        });
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Err(ServiceError::Forbidden));
    let service = dependencies.build_service();

    let result = service
        .get_by_week_for_sales_person(default_sales_person_id(), 2063, 42, ().into())
        .await;
    test_forbidden(&result);
}

#[tokio::test]
pub async fn test_create_shiftplanner() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies
        .sales_person_unavailable_dao
        .expect_create()
        .with(
            eq(SalesPersonUnavailableEntity {
                ..default_sales_person_unavailable_entity()
            }),
            eq("SalesPersonUnavailableService::create"),
        )
        .returning(|_, _| Ok(()));
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Err(ServiceError::Forbidden));
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_week_and_sales_person_id()
        .with(eq(default_sales_person_id()), eq(2063), eq(42))
        .returning(|_, _, _| Ok([].into()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("SalesPersonUnavailableService::create id"))
        .returning(|_| default_id());
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("SalesPersonUnavailableService::create version"))
        .returning(|_| default_version());
    let service = dependencies.build_service();

    let result = service
        .create(
            &SalesPersonUnavailable {
                id: Uuid::nil(),
                version: Uuid::nil(),
                created: None,
                ..default_sales_person_unavailable()
            },
            ().into(),
        )
        .await
        .unwrap();
    assert_eq!(result, default_sales_person_unavailable());
}

#[tokio::test]
pub async fn test_create_id_set() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Err(ServiceError::Forbidden));
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_week_and_sales_person_id()
        .with(eq(default_sales_person_id()), eq(2063), eq(42))
        .returning(|_, _, _| Ok([].into()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("SalesPersonUnavailableService::create id"))
        .returning(|_| default_id());
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("SalesPersonUnavailableService::create version"))
        .returning(|_| default_version());
    let service = dependencies.build_service();

    let result = service
        .create(
            &SalesPersonUnavailable {
                version: Uuid::nil(),
                created: None,
                ..default_sales_person_unavailable()
            },
            ().into(),
        )
        .await;
    test_zero_id_error(&result);
}

#[tokio::test]
pub async fn test_create_version_set() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Err(ServiceError::Forbidden));
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_week_and_sales_person_id()
        .with(eq(default_sales_person_id()), eq(2063), eq(42))
        .returning(|_, _, _| Ok([].into()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("SalesPersonUnavailableService::create id"))
        .returning(|_| default_id());
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("SalesPersonUnavailableService::create version"))
        .returning(|_| default_version());
    let service = dependencies.build_service();

    let result = service
        .create(
            &SalesPersonUnavailable {
                id: Uuid::nil(),
                created: None,
                ..default_sales_person_unavailable()
            },
            ().into(),
        )
        .await;
    test_zero_version_error(&result);
}

#[tokio::test]
pub async fn test_create_already_exists() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Err(ServiceError::Forbidden));
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_week_and_sales_person_id()
        .with(eq(default_sales_person_id()), eq(2063), eq(42))
        .returning(|_, _, _| Ok([default_sales_person_unavailable_entity()].into()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("SalesPersonUnavailableService::create id"))
        .returning(|_| default_id());
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("SalesPersonUnavailableService::create version"))
        .returning(|_| default_version());
    let service = dependencies.build_service();

    let result = service
        .create(
            &SalesPersonUnavailable {
                id: Uuid::nil(),
                version: Uuid::nil(),
                created: None,
                ..default_sales_person_unavailable()
            },
            ().into(),
        )
        .await;
    test_exists_error(&result, default_id());
}

#[tokio::test]
pub async fn test_create_sales_person() {
    let mut dependencies = build_dependencies(true, "sales");
    dependencies
        .sales_person_unavailable_dao
        .expect_create()
        .with(
            eq(SalesPersonUnavailableEntity {
                ..default_sales_person_unavailable_entity()
            }),
            eq("SalesPersonUnavailableService::create"),
        )
        .returning(|_, _| Ok(()));
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Ok(()));
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_week_and_sales_person_id()
        .with(eq(default_sales_person_id()), eq(2063), eq(42))
        .returning(|_, _, _| Ok([].into()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("SalesPersonUnavailableService::create id"))
        .returning(|_| default_id());
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("SalesPersonUnavailableService::create version"))
        .returning(|_| default_version());
    let service = dependencies.build_service();

    service
        .create(
            &SalesPersonUnavailable {
                id: Uuid::nil(),
                version: Uuid::nil(),
                created: None,
                ..default_sales_person_unavailable()
            },
            ().into(),
        )
        .await
        .unwrap();
}

#[tokio::test]
pub async fn test_create_no_permission() {
    let mut dependencies = build_dependencies(false, "sales");
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Err(ServiceError::Forbidden));
    let service = dependencies.build_service();

    let result = service
        .create(&default_sales_person_unavailable(), ().into())
        .await;
    test_forbidden(&result);
}

#[tokio::test]
pub async fn test_delete_shiftplanner() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Err(ServiceError::Forbidden));
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(default_sales_person_unavailable_entity())));
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(None));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("SalesPersonUnavailableService::delete version"))
        .returning(|_| alternate_version());
    dependencies
        .sales_person_unavailable_dao
        .expect_update()
        .with(
            eq(SalesPersonUnavailableEntity {
                deleted: Some(time::PrimitiveDateTime::new(
                    time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
                    time::Time::from_hms(23, 42, 0).unwrap(),
                )),
                version: alternate_version(),
                ..default_sales_person_unavailable_entity()
            }),
            eq("SalesPersonUnavailableService::delete"),
        )
        .returning(|_, _| Ok(()));
    let service = dependencies.build_service();

    service.delete(default_id(), ().into()).await.unwrap();
}

#[tokio::test]
pub async fn test_delete_sales_person() {
    let mut dependencies = build_dependencies(true, "sales");
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Ok(()));
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(default_sales_person_unavailable_entity())));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("SalesPersonUnavailableService::delete version"))
        .returning(|_| alternate_version());
    dependencies
        .sales_person_unavailable_dao
        .expect_update()
        .with(
            eq(SalesPersonUnavailableEntity {
                deleted: Some(time::PrimitiveDateTime::new(
                    time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
                    time::Time::from_hms(23, 42, 0).unwrap(),
                )),
                version: alternate_version(),
                ..default_sales_person_unavailable_entity()
            }),
            eq("SalesPersonUnavailableService::delete"),
        )
        .returning(|_, _| Ok(()));
    let service = dependencies.build_service();

    service.delete(default_id(), ().into()).await.unwrap();
}

#[tokio::test]
pub async fn test_delete_no_permission() {
    let mut dependencies = build_dependencies(false, "sales");
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(default_sales_person_unavailable_entity())));
    dependencies
        .sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _| Err(ServiceError::Forbidden));
    let service = dependencies.build_service();

    let result = service.delete(default_id(), ().into()).await;
    test_forbidden(&result);
}

#[tokio::test]
pub async fn test_delete_not_found() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(None));
    dependencies
        .sales_person_unavailable_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(None));
    let service = dependencies.build_service();

    let result = service.delete(default_id(), ().into()).await;
    test_not_found(&result, &default_id());
}
