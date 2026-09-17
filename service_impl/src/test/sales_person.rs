use super::error_test::*;
use dao::{
    sales_person::{MockSalesPersonDao, SalesPersonEntity},
    DaoError, MockTransaction, MockTransactionDao,
};
use mockall::predicate::{always, eq};
use service::{
    clock::MockClockService,
    permission::Authentication,
    sales_person::{SalesPerson, SalesPersonService},
    uuid_service::MockUuidService,
    MockPermissionService, ServiceError,
};
use time::{Date, Month, PrimitiveDateTime, Time};
use tokio;
use uuid::{uuid, Uuid};

use crate::sales_person::{SalesPersonServiceDeps, SalesPersonServiceImpl};

pub struct SalesPersonServiceDependencies {
    pub sales_person_dao: MockSalesPersonDao,
    pub permission_service: MockPermissionService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
    pub transaction_dao: MockTransactionDao,
}
impl SalesPersonServiceDeps for SalesPersonServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type SalesPersonDao = MockSalesPersonDao;
    type PermissionService = MockPermissionService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = MockTransactionDao;
}

impl SalesPersonServiceDependencies {
    pub fn build_service(self) -> SalesPersonServiceImpl<SalesPersonServiceDependencies> {
        SalesPersonServiceImpl {
            sales_person_dao: self.sales_person_dao.into(),
            permission_service: self.permission_service.into(),
            clock_service: self.clock_service.into(),
            uuid_service: self.uuid_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
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

    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    SalesPersonServiceDependencies {
        sales_person_dao,
        permission_service,
        clock_service,
        uuid_service,
        transaction_dao,
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
    dependencies.sales_person_dao.expect_all().returning(|_| {
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
    let result = sales_person_service.get_all(().auth(), None).await.unwrap();
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
    dependencies.sales_person_dao.expect_all().returning(|_| {
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
    let result = sales_person_service.get_all(().auth(), None).await.unwrap();
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
    dependencies.sales_person_dao.expect_all().returning(|_| {
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
    let result = sales_person_service.get_all(().auth(), None).await.unwrap();
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
    let result = sales_person_service.get_all(().auth(), None).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_get_hr_user() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .times(1)
        .returning(|_, _| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_get_assigned_user()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some("TESTUSER".into())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .get(default_id(), ().auth(), None)
        .await;
    assert_eq!(default_sales_person(), result.unwrap());
}

#[tokio::test]
async fn test_get_shiftplanner_user_other_user() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .times(1)
        .returning(|_, _| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_get_assigned_user()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some("OTHER".into())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .get(default_id(), ().auth(), None)
        .await;
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
        .with(eq(default_id()), always())
        .times(1)
        .returning(|_, _| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_get_assigned_user()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some("OTHER".into())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .get(default_id(), ().auth(), None)
        .await;
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
        .with(eq(default_id()), always())
        .times(1)
        .returning(|_, _| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_get_assigned_user()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some("TESTUSER".into())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .get(default_id(), ().auth(), None)
        .await;
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
        .with(eq(default_id()), always())
        .times(1)
        .returning(|_, _| Ok(Some(default_sales_person_entity())));
    dependencies
        .sales_person_dao
        .expect_get_assigned_user()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some("TESTUSER".into())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .get(default_id(), ().auth(), None)
        .await;
    assert_eq!(default_sales_person(), result.unwrap());
}

#[tokio::test]
async fn test_get_no_permission() {
    let dependencies = build_dependencies(false, "sales");
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .get(default_id(), ().auth(), None)
        .await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_get_not_found() {
    let mut dependencies = build_dependencies(true, "shiftplanner");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .times(1)
        .returning(|_, _| Ok(None));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .get(default_id(), ().auth(), None)
        .await;
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_get_not_found_sales_user() {
    let mut dependencies = build_dependencies(true, "sales");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .times(1)
        .returning(|_, _| Ok(None));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .get(default_id(), ().auth(), None)
        .await;
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
            always(),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));
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
            None,
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
            None,
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
            None,
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
            None,
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
            None,
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
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(None));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .update(
            &SalesPerson {
                name: "Jane Doe".into(),
                ..default_sales_person()
            },
            ().auth(),
            None,
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
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_sales_person_entity())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .update(
            &SalesPerson {
                version: alternate_version(),
                ..default_sales_person()
            },
            ().auth(),
            None,
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
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_sales_person_entity())));
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
            None,
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
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_sales_person_entity())));
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
            always(),
        )
        .returning(|_, _, _| Ok(()));
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
            None,
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
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_sales_person_entity())));
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
            always(),
        )
        .returning(|_, _, _| Ok(()));
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
            None,
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
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_sales_person_entity())));
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
            always(),
        )
        .returning(|_, _, _| Ok(()));
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
            None,
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
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_sales_person_entity())));
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
            always(),
        )
        .returning(|_, _, _| Ok(()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("sales-person-version"))
        .returning(|_| alternate_version());
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .delete(default_id(), ().auth(), None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_no_permission() {
    let mut dependencies = build_dependencies(false, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_sales_person_entity())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .delete(default_id(), ().auth(), None)
        .await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_delete_not_found() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(None));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .delete(default_id(), ().auth(), None)
        .await;
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_exists() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(Some(default_sales_person_entity())));
    let sales_person_service = dependencies.build_service();
    let result = sales_person_service
        .exists(default_id(), ().auth(), None)
        .await
        .unwrap();
    assert!(result);

    let mut dependencies = build_dependencies(true, "hr");
    dependencies.sales_person_dao.checkpoint();
    dependencies
        .sales_person_dao
        .expect_find_by_id()
        .with(eq(default_id()), always())
        .returning(|_, _| Ok(None));
    let result = sales_person_service
        .exists(default_id(), ().auth(), None)
        .await
        .unwrap();
    assert!(result);
}

// --- WR-04: Fehleruebersetzung des Split-Pfads in `set_user` -----------------
//
// Herkunft: `61-REVIEW.md` Abschnitt WR-04. `discard_assigned_user` liefert bei einem
// Race-Verlierer (`rows_affected() != 1`) `DaoError::EntityAlreadyExists` -
// denselben Fehler, den `assign_to_user` im Konfliktfall liefert. Geprueft wird
// hier ausschliesslich die **Uebersetzung im Service**, nicht das SQL.
//
// Bewusst Mock-basiert statt als Integrationstest: der
// `rows_affected() != 1`-Zweig in `discard_assigned_user` ist ueber eine echte
// SQLite nicht deterministisch herstellbar, weil die Vorab-Lesung und das
// `UPDATE` in derselben Transaktion laufen und kein Nebenlaeufer dazwischen
// kommt. Der Mock stellt genau den Rueckgabewert her, den die echte
// Implementierung in diesem Zweig liefert. Das ist keine Abweichung vom
// Integrations-Stil der Person-Testmodule (`person_merge`, `person_creation`),
// sondern die einzige Bauart, in der dieser Zweig ueberhaupt erreichbar ist.

/// WR-04: Ein Split-Konflikt muss den Aufrufer als
/// `ServiceError::EntityAlreadyExists` (HTTP 409) erreichen, nicht als
/// Serverfehler.
///
/// Mock statt Integration: siehe Abschnittskommentar oben - der
/// `rows_affected() != 1`-Zweig ist in einer einzigen Transaktion nicht
/// deterministisch erzeugbar.
#[tokio::test]
async fn set_user_none_uebersetzt_split_konflikt_nach_409() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_discard_assigned_user()
        .with(eq(default_id()), always())
        .times(1)
        .returning(|_, _| Err(DaoError::EntityAlreadyExists));
    // `expect_assign_to_user` wird bewusst NICHT gesetzt: `set_user(_, None, ..)`
    // darf den Merge-Pfad gar nicht erreichen. mockall meldet einen
    // unerwarteten Aufruf von selbst.
    let sales_person_service = dependencies.build_service();

    let result = sales_person_service
        .set_user(default_id(), None, ().auth(), None)
        .await;

    match &result {
        Err(ServiceError::EntityAlreadyExists(id)) => {
            assert_eq!(
                &default_id(),
                id,
                "Der Konflikt muss die angefragte Sales Person nennen"
            );
        }
        other => panic!(
            "Split-Konflikt wurde nicht als Konflikt gemeldet: {:?}. \
             Dieselbe DAO-Bedingung darf nicht auf POST /sales-person/{{id}}/user \
             einen 409 und auf DELETE /sales-person/{{id}}/user einen 500 ergeben \
             - DELETE faehrt ausschliesslich den Split (WR-04).",
            other
        ),
    }
}

/// Grenze der Uebersetzung aus WR-04: ein anderer `DaoError` im Split bleibt
/// ein Serverfehler. Verhindert, dass der Fix zu einem pauschalen
/// "jeder DAO-Fehler im Split wird 409" ausartet und damit einen echten Defekt
/// als fachlichen Konflikt tarnt (T-61-32).
///
/// Mock statt Integration: siehe Abschnittskommentar oben.
#[tokio::test]
async fn set_user_none_laesst_andere_dao_fehler_als_serverfehler_durch() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .sales_person_dao
        .expect_discard_assigned_user()
        .with(eq(default_id()), always())
        .times(1)
        .returning(|_, _| {
            Err(DaoError::DatabaseQueryError(
                "simulierter Serverfehler im Split".into(),
            ))
        });
    let sales_person_service = dependencies.build_service();

    let result = sales_person_service
        .set_user(default_id(), None, ().auth(), None)
        .await;

    assert!(
        !matches!(&result, Err(ServiceError::EntityAlreadyExists(_))),
        "Ein beliebiger DAO-Fehler darf nicht als Konflikt getarnt werden: {:?}",
        result
    );
    assert!(
        matches!(&result, Err(ServiceError::DatabaseQueryError(_))),
        "Ein beliebiger DAO-Fehler muss unveraendert als Serverfehler durchgehen: {:?}",
        result
    );
}
