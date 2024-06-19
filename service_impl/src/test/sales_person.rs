use super::error_test::*;
use dao::sales_person::{MockSalesPersonDao, SalesPersonEntity};
use mockall::predicate::{always, eq};
use service::{
    clock::MockClockService,
    permission::Authentication,
    sales_person::{SalesPerson, SalesPersonService},
    uuid_service::MockUuidService,
    MockPermissionService,
};
use time::{Date, Month, PrimitiveDateTime, Time};
use tokio;
use uuid::{uuid, Uuid};

use crate::sales_person::SalesPersonServiceImpl;

pub struct SalesPersonServiceDependencies {
    pub sales_person_dao: MockSalesPersonDao,
    pub permission_service: MockPermissionService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
}
impl SalesPersonServiceDependencies {
    pub fn build_service(
        self,
    ) -> SalesPersonServiceImpl<
        MockSalesPersonDao,
        MockPermissionService,
        MockClockService,
        MockUuidService,
    > {
        SalesPersonServiceImpl::new(
            self.sales_person_dao.into(),
            self.permission_service.into(),
            self.clock_service.into(),
            self.uuid_service.into(),
        )
    }
}

pub fn build_dependencies(permission: bool, role: &'static str) -> SalesPersonServiceDependencies {
    let sales_person_dao = MockSalesPersonDao::new();
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

    SalesPersonServiceDependencies {
        sales_person_dao,
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

pub fn default_sales_person_entity() -> dao::sales_person::SalesPersonEntity {
    dao::sales_person::SalesPersonEntity {
        id: default_id(),
        name: "John Doe".into(),
        background_color: "#FFF".into(),
        is_paid: false,
        deleted: None,
        inactive: false,
        version: default_version(),
    }
}

pub fn default_sales_person() -> service::sales_person::SalesPerson {
    service::sales_person::SalesPerson {
        id: default_id(),
        name: "John Doe".into(),
        background_color: "#FFF".into(),
        is_paid: Some(false),
        inactive: false,
        deleted: None,
        version: default_version(),
    }
}

#[tokio::test]
async fn test_get_all_shiftplanner() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies.sales_person_dao.expect_all().returning(|| {
        Ok([
            default_sales_person_entity(),
            SalesPersonEntity {
                id: alternate_id(),
                name: "Jane Doe".into(),
                ..default_sales_person_entity()
            },
        ]
        .into())
    });
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.get_all(().auth()).await.unwrap();
    assert_eq!(2, result.len());
    assert_eq!(
        service::sales_person::SalesPerson {
            is_paid: None,
            ..default_sales_person()
        },
        result[0]
    );
    assert_eq!(
        service::sales_person::SalesPerson {
            id: alternate_id(),
            name: "Jane Doe".into(),
            is_paid: None,
            ..default_sales_person()
        },
        result[1]
    );
}

#[tokio::test]
async fn test_get_all_sales_user() {
    let mut dependencies = build_dependencies(true, "sales");
    dependencies.sales_person_dao.expect_all().returning(|| {
        Ok([
            default_sales_person_entity(),
            SalesPersonEntity {
                id: alternate_id(),
                name: "Jane Doe".into(),
                ..default_sales_person_entity()
            },
        ]
        .into())
    });
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.get_all(().auth()).await.unwrap();
    assert_eq!(2, result.len());
    assert_eq!(
        service::sales_person::SalesPerson {
            is_paid: None,
            ..default_sales_person()
        },
        result[0]
    );
    assert_eq!(
        service::sales_person::SalesPerson {
            id: alternate_id(),
            name: "Jane Doe".into(),
            is_paid: None,
            ..default_sales_person()
        },
        result[1]
    );
}

#[tokio::test]
async fn test_get_all_hr_user() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies.sales_person_dao.expect_all().returning(|| {
        Ok([
            default_sales_person_entity(),
            SalesPersonEntity {
                id: alternate_id(),
                name: "Jane Doe".into(),
                ..default_sales_person_entity()
            },
        ]
        .into())
    });
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.get_all(().auth()).await.unwrap();
    assert_eq!(2, result.len());
    assert_eq!(default_sales_person(), result[0]);
    assert_eq!(
        service::sales_person::SalesPerson {
            id: alternate_id(),
            name: "Jane Doe".into(),
            ..default_sales_person()
        },
        result[1]
    );
}

#[tokio::test]
async fn test_get_all_no_permission() {
    let dependencies = build_dependencies(false, "hr");
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.get_all(().auth()).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_get_hr_user() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_get_assigned_user()
        .with(eq(default_id()))
        .returning(|_| Ok(Some("TESTUSER".into())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.get(default_id(), ().auth()).await;
    assert_eq!(default_sales_person(), result.unwrap());
}

#[tokio::test]
async fn test_get_shiftplanner_user_other_user() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_get_assigned_user()
        .with(eq(default_id()))
        .returning(|_| Ok(Some("OTHER".into())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.get(default_id(), ().auth()).await;
    assert_eq!(
        SalesPerson {
            is_paid: None,
            ..default_sales_person()
        },
        result.unwrap()
    );
}

#[tokio::test]
async fn test_get_sales_user_other_user() {
    let mut dependencies = build_dependencies(true, "sales");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_get_assigned_user()
        .with(eq(default_id()))
        .returning(|_| Ok(Some("OTHER".into())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.get(default_id(), ().auth()).await;
    assert_eq!(
        SalesPerson {
            is_paid: None,
            ..default_sales_person()
        },
        result.unwrap()
    );
}

#[tokio::test]
async fn test_get_shiftplanner_user_same_user() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_get_assigned_user()
        .with(eq(default_id()))
        .returning(|_| Ok(Some("TESTUSER".into())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.get(default_id(), ().auth()).await;
    assert_eq!(
        SalesPerson {
            ..default_sales_person()
        },
        result.unwrap()
    );
}

#[tokio::test]
async fn test_get_sales_user_same_user() {
    let mut dependencies = build_dependencies(true, "sales");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_get_assigned_user()
        .with(eq(default_id()))
        .returning(|_| Ok(Some("TESTUSER".into())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.get(default_id(), ().auth()).await;
    assert_eq!(default_sales_person(), result.unwrap());
}

#[tokio::test]
async fn test_get_no_permission() {
    let dependencies = build_dependencies(false, "sales");
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.get(default_id(), ().auth()).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_get_not_found() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(None));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.get(default_id(), ().auth()).await;
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_get_not_found_sales_user() {
    let mut dependencies = build_dependencies(true, "sales");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(None));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.get(default_id(), ().auth()).await;
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_create() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_create()
        .with(
            eq(default_sales_person_entity()),
            eq("sales-person-service"),
        )
        .times(1)
        .returning(|_, _| Ok(()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("sales-person-id"))
        .times(1)
        .returning(|_| default_id());
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("sales-person-version"))
        .times(1)
        .returning(|_| default_version());
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .create(
            &SalesPerson {
                id: Uuid::nil(),
                version: Uuid::nil(),
                ..default_sales_person()
            },
            ().auth(),
        )
        .await
        .unwrap();
    assert_eq!(result, default_sales_person());
}

#[tokio::test]
async fn test_create_no_permission() {
    let dependencies = build_dependencies(false, "hr");
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .create(
            &SalesPerson {
                id: Uuid::nil(),
                version: Uuid::nil(),
                ..default_sales_person()
            },
            ().auth(),
        )
        .await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_create_validation() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("sales-person-id"))
        .returning(|_| default_id());
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("sales-person-version"))
        .returning(|_| default_version());

    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .create(
            &SalesPerson {
                version: Uuid::nil(),
                ..default_sales_person()
            },
            ().auth(),
        )
        .await;
    test_zero_id_error(&result);

    let result = sales_person_service
        .create(
            &SalesPerson {
                id: Uuid::nil(),
                ..default_sales_person()
            },
            ().auth(),
        )
        .await;
    test_zero_version_error(&result);
}

#[tokio::test]
async fn test_update_no_permission() {
    let dependencies = build_dependencies(false, "hr");
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .update(
            &SalesPerson {
                name: "Jane Doe".into(),
                ..default_sales_person()
            },
            ().auth(),
        )
        .await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_update_not_found() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(None));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .update(
            &SalesPerson {
                name: "Jane Doe".into(),
                ..default_sales_person()
            },
            ().auth(),
        )
        .await;
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_update_conflicts() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(default_sales_person_entity())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .update(
            &SalesPerson {
                version: alternate_version(),
                ..default_sales_person()
            },
            ().auth(),
        )
        .await;
    test_conflicts(
        &result,
        &default_id(),
        &default_version(),
        &alternate_version(),
    );
}

#[tokio::test]
async fn test_update_deleted_no_allowed() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(default_sales_person_entity())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .update(
            &SalesPerson {
                deleted: Some(PrimitiveDateTime::new(
                    Date::from_calendar_date(2000, Month::January, 1).unwrap(),
                    Time::from_hms(1, 0, 0).unwrap(),
                )),
                ..default_sales_person()
            },
            ().auth(),
        )
        .await;
    test_validation_error(
        &result,
        &service::ValidationFailureItem::ModificationNotAllowed("deleted".into()),
        1,
    );
}

#[tokio::test]
async fn test_update_inactive() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_update()
        .with(
            eq(SalesPersonEntity {
                inactive: true,
                version: alternate_version(),
                ..default_sales_person_entity()
            }),
            eq("sales-person-service"),
        )
        .returning(|_, _| Ok(()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("sales-person-version"))
        .returning(|_| alternate_version());
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .update(
            &SalesPerson {
                inactive: true,
                ..default_sales_person()
            },
            ().auth(),
        )
        .await
        .unwrap();
    assert_eq!(
        result,
        SalesPerson {
            inactive: true,
            version: alternate_version(),
            ..default_sales_person()
        }
    );
}

#[tokio::test]
async fn test_update_name() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_update()
        .with(
            eq(SalesPersonEntity {
                name: "Jane Doe".into(),
                version: alternate_version(),
                ..default_sales_person_entity()
            }),
            eq("sales-person-service"),
        )
        .returning(|_, _| Ok(()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("sales-person-version"))
        .returning(|_| alternate_version());
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .update(
            &SalesPerson {
                name: "Jane Doe".into(),
                ..default_sales_person()
            },
            ().auth(),
        )
        .await
        .unwrap();
    assert_eq!(
        result,
        SalesPerson {
            name: "Jane Doe".into(),
            version: alternate_version(),
            ..default_sales_person()
        }
    );
}

#[tokio::test]
async fn test_update_background_color() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_update()
        .with(
            eq(SalesPersonEntity {
                background_color: "#000".into(),
                version: alternate_version(),
                ..default_sales_person_entity()
            }),
            eq("sales-person-service"),
        )
        .returning(|_, _| Ok(()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("sales-person-version"))
        .returning(|_| alternate_version());
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .update(
            &SalesPerson {
                background_color: "#000".into(),
                ..default_sales_person()
            },
            ().auth(),
        )
        .await
        .unwrap();
    assert_eq!(
        result,
        SalesPerson {
            background_color: "#000".into(),
            version: alternate_version(),
            ..default_sales_person()
        }
    );
}

#[tokio::test]
async fn test_delete() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_update()
        .with(
            eq(SalesPersonEntity {
                deleted: Some(PrimitiveDateTime::new(
                    Date::from_calendar_date(2063, Month::April, 5).unwrap(),
                    Time::from_hms(23, 42, 0).unwrap(),
                )),
                version: alternate_version(),
                ..default_sales_person_entity()
            }),
            eq("sales-person-service"),
        )
        .returning(|_, _| Ok(()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("sales-person-version"))
        .returning(|_| alternate_version());
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.delete(default_id(), ().auth()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_no_permission() {
    let mut dependencies = build_dependencies(false, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(default_sales_person_entity())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.delete(default_id(), ().auth()).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_delete_not_found() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(None));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service.delete(default_id(), ().auth()).await;
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_exists() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(default_sales_person_entity())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .exists(default_id(), ().auth())
        .await
        .unwrap();
    assert!(result);

    let mut dependencies = build_dependencies(true, "hr");
    dependencies.sales_person_dao.checkpoint();
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()))
        .returning(|_| Ok(None));
    let result = sales_person_service
        .exists(default_id(), ().auth())
        .await
        .unwrap();
    assert_eq!(false, !result);
}
