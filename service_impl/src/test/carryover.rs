use std::sync::Arc;

use crate::carryover::{CarryoverServiceDeps, CarryoverServiceImpl};
use dao::carryover::CarryoverEntity;
use dao::{MockTransaction, MockTransactionDao};
use mockall::predicate::{always, eq};
use service::carryover::{Carryover, CarryoverService};
use service::permission::Authentication;
use service::ServiceError;
use time::{Date, Month, PrimitiveDateTime, Time};
use uuid::{uuid, Uuid};

use dao::carryover::MockCarryoverDao;

// Helper function to create a default Carryover entity
fn default_carryover_entity() -> CarryoverEntity {
    CarryoverEntity {
        sales_person_id: default_sales_person_id(),
        year: 2025,
        carryover_hours: 10.0,
        vacation: 1,
        created: generate_default_datetime(),
        deleted: None,
        version: default_version(),
    }
}

// Helper function to create a default Carryover struct
fn default_carryover() -> Carryover {
    Carryover {
        sales_person_id: default_sales_person_id(),
        year: 2025,
        carryover_hours: 10.0,
        vacation: 1,
        created: generate_default_datetime(),
        deleted: None,
        version: default_version(),
    }
}

fn default_sales_person_id() -> Uuid {
    uuid!("04215DFE-13C4-413C-8C66-77AC741BB5F0")
}

fn default_version() -> Uuid {
    uuid!("F79C462A-8D4E-42E1-8171-DB4DBD019E50")
}

fn generate_default_datetime() -> PrimitiveDateTime {
    PrimitiveDateTime::new(
        Date::from_calendar_date(2024, Month::January, 1).unwrap(),
        Time::from_hms(0, 0, 0).unwrap(),
    )
}

// Dependencies for the Carryover service
pub struct CarryoverServiceDependencies {
    pub carryover_dao: MockCarryoverDao,
}

impl CarryoverServiceDeps for CarryoverServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;

    type CarryoverDao = MockCarryoverDao;

    type TransactionDao = MockTransactionDao;
}

impl CarryoverServiceDependencies {
    pub fn build_service(self) -> CarryoverServiceImpl<CarryoverServiceDependencies> {
        let mut transaction_dao = MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        CarryoverServiceImpl {
            carryover_dao: self.carryover_dao.into(),
            transaction_dao: Arc::new(transaction_dao),
        }
    }
}

fn build_dependencies() -> CarryoverServiceDependencies {
    // By default no expectations, tests will set them up
    CarryoverServiceDependencies {
        carryover_dao: MockCarryoverDao::new(),
    }
}

// Since CarryoverService doesn't do permission checks, we pass ()
// and convert it to Authentication::Context(())
trait NoneTypeExt {
    fn auth(&self) -> Authentication<()>;
}
impl NoneTypeExt for () {
    fn auth(&self) -> Authentication<()> {
        Authentication::Context(())
    }
}

#[tokio::test]
async fn test_get_carryover_found() {
    let mut deps = build_dependencies();
    let entity = default_carryover_entity();
    deps.carryover_dao
        .expect_find_by_sales_person_id_and_year()
        .with(eq(entity.sales_person_id), eq(entity.year), always())
        .returning(move |_, _, _| Ok(Some(entity.clone())));

    let service = deps.build_service();
    let result = service
        .get_carryover(default_sales_person_id(), 2025, ().auth(), None)
        .await;
    assert!(result.is_ok(), "Expected Ok result");
    let carryover = result.unwrap().expect("Expected Some carryover");
    assert_eq!(carryover, default_carryover());
}

#[tokio::test]
async fn test_get_carryover_not_found() {
    let mut deps = build_dependencies();
    deps.carryover_dao
        .expect_find_by_sales_person_id_and_year()
        .with(eq(default_sales_person_id()), eq(2025), always())
        .returning(|_, _, _| Ok(None));

    let service = deps.build_service();
    let result = service
        .get_carryover(default_sales_person_id(), 2025, ().auth(), None)
        .await;
    assert!(result.is_ok(), "Expected Ok result even if not found");
    assert!(result.unwrap().is_none(), "Expected None for not found");
}

#[tokio::test]
async fn test_get_carryover_dao_error() {
    let mut deps = build_dependencies();
    deps.carryover_dao
        .expect_find_by_sales_person_id_and_year()
        .with(eq(default_sales_person_id()), eq(2025), always())
        .returning(|_, _, _| Err(dao::DaoError::DatabaseQueryError("Some DB error".into())));

    let service = deps.build_service();
    let result = service
        .get_carryover(default_sales_person_id(), 2025, ().auth(), None)
        .await;
    match result {
        Err(ServiceError::DatabaseQueryError(_)) => { /* expected */ }
        _ => panic!("Expected a data access error"),
    }
}

#[tokio::test]
async fn test_set_carryover_success() {
    let mut deps = build_dependencies();
    let carryover = default_carryover();
    let entity = default_carryover_entity();

    deps.carryover_dao
        .expect_upsert()
        .with(eq(entity.clone()), eq("carryover-service"), always())
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let result = service.set_carryover(&carryover, ().auth(), None).await;
    assert!(result.is_ok(), "Expected Ok result");
}

#[tokio::test]
async fn test_set_carryover_dao_error() {
    let mut deps = build_dependencies();
    let carryover = default_carryover();

    deps.carryover_dao
        .expect_upsert()
        .with(always(), eq("carryover-service"), always())
        .returning(|_, _, _| Err(dao::DaoError::DatabaseQueryError("DB issue".into())));

    let service = deps.build_service();
    let result = service.set_carryover(&carryover, ().auth(), None).await;
    match result {
        Err(ServiceError::DatabaseQueryError(_)) => { /* expected */ }
        _ => panic!("Expected a data access error"),
    }
}
