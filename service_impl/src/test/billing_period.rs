use std::sync::Arc;

use mockall::predicate::{self, *};
use service::billing_period::BillingPeriodService;
use service::permission::Authentication;
use service::ServiceError;
use time::macros::datetime;
use uuid::Uuid;

use crate::billing_period::{BillingPeriodServiceDeps, BillingPeriodServiceImpl};

struct MockDeps {
    billing_period_dao: dao::billing_period::MockBillingPeriodDao,
    billing_period_sales_person_dao:
        dao::billing_period_sales_person::MockBillingPeriodSalesPersonDao,
    sales_person_service: service::sales_person::MockSalesPersonService,
    permission_service: service::MockPermissionService,
    uuid_service: service::uuid_service::MockUuidService,
    clock_service: service::clock::MockClockService,
    transaction_dao: dao::MockTransactionDao,
}

impl BillingPeriodServiceDeps for MockDeps {
    type Context = ();
    type Transaction = dao::MockTransaction;
    type BillingPeriodDao = dao::billing_period::MockBillingPeriodDao;
    type BillingPeriodSalesPersonDao =
        dao::billing_period_sales_person::MockBillingPeriodSalesPersonDao;
    type SalesPersonService = service::sales_person::MockSalesPersonService;
    type PermissionService = service::MockPermissionService;
    type UuidService = service::uuid_service::MockUuidService;
    type ClockService = service::clock::MockClockService;
    type TransactionDao = dao::MockTransactionDao;
}

impl MockDeps {
    fn new() -> Self {
        Self {
            billing_period_dao: dao::billing_period::MockBillingPeriodDao::new(),
            billing_period_sales_person_dao:
                dao::billing_period_sales_person::MockBillingPeriodSalesPersonDao::new(),
            sales_person_service: service::sales_person::MockSalesPersonService::new(),
            permission_service: service::MockPermissionService::new(),
            uuid_service: service::uuid_service::MockUuidService::new(),
            clock_service: service::clock::MockClockService::new(),
            transaction_dao: dao::MockTransactionDao::new(),
        }
    }

    fn build_service(self) -> BillingPeriodServiceImpl<MockDeps> {
        BillingPeriodServiceImpl {
            billing_period_dao: self.billing_period_dao.into(),
            billing_period_sales_person_dao: self.billing_period_sales_person_dao.into(),
            sales_person_service: self.sales_person_service.into(),
            permission_service: self.permission_service.into(),
            uuid_service: self.uuid_service.into(),
            clock_service: self.clock_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

fn make_billing_period_entity(
    id: Uuid,
    start: time::Date,
    end: time::Date,
) -> dao::billing_period::BillingPeriodEntity {
    dao::billing_period::BillingPeriodEntity {
        id,
        start_date: start,
        end_date: end,
        snapshot_schema_version: 1,
        created_at: datetime!(2024-01-01 0:00),
        created_by: "test".into(),
        deleted_at: None,
        deleted_by: None,
    }
}

fn setup_common_mocks(deps: &mut MockDeps) {
    deps.transaction_dao
        .expect_use_transaction()
        .with(predicate::always())
        .times(1)
        .returning(|_| Ok(dao::MockTransaction));
}

#[tokio::test]
async fn test_delete_billing_period_success() {
    let latest_id = Uuid::new_v4();
    let older_id = Uuid::new_v4();
    let context = Authentication::Full;

    let mut deps = MockDeps::new();
    setup_common_mocks(&mut deps);

    deps.permission_service
        .expect_check_permission()
        .with(
            eq(service::permission::HR_PRIVILEGE),
            eq(context.clone()),
        )
        .times(1)
        .returning(|_, _| Ok(()));

    let entity_latest = make_billing_period_entity(
        latest_id,
        time::macros::date!(2025 - 04 - 01),
        time::macros::date!(2025 - 06 - 30),
    );
    let entity_older = make_billing_period_entity(
        older_id,
        time::macros::date!(2025 - 01 - 01),
        time::macros::date!(2025 - 03 - 31),
    );

    let entity_latest_clone = entity_latest.clone();
    deps.billing_period_dao
        .expect_find_by_id()
        .with(eq(latest_id), always())
        .times(1)
        .returning(move |_, _| Ok(Some(entity_latest_clone.clone())));

    let all_entities: Arc<[dao::billing_period::BillingPeriodEntity]> =
        vec![entity_latest.clone(), entity_older.clone()].into();
    deps.billing_period_dao
        .expect_all_ordered_desc()
        .with(always())
        .times(1)
        .returning(move |_| Ok(all_entities.clone()));

    deps.permission_service
        .expect_current_user_id()
        .with(always())
        .times(1)
        .returning(|_| Ok(Some("test_user".into())));

    deps.billing_period_sales_person_dao
        .expect_delete_by_billing_period_id()
        .with(eq(latest_id), eq("test_user"), always())
        .times(1)
        .returning(|_, _, _| Ok(()));

    deps.billing_period_dao
        .expect_delete_by_id()
        .with(eq(latest_id), eq("test_user"), always())
        .times(1)
        .returning(|_, _, _| Ok(()));

    deps.transaction_dao
        .expect_commit()
        .with(always())
        .times(1)
        .returning(|_| Ok(()));

    let service = deps.build_service();
    let result = service
        .delete_billing_period(latest_id, context, None)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_billing_period_not_latest_returns_409() {
    let latest_id = Uuid::new_v4();
    let older_id = Uuid::new_v4();
    let context = Authentication::Full;

    let mut deps = MockDeps::new();
    setup_common_mocks(&mut deps);

    deps.permission_service
        .expect_check_permission()
        .with(
            eq(service::permission::HR_PRIVILEGE),
            eq(context.clone()),
        )
        .times(1)
        .returning(|_, _| Ok(()));

    let entity_latest = make_billing_period_entity(
        latest_id,
        time::macros::date!(2025 - 04 - 01),
        time::macros::date!(2025 - 06 - 30),
    );
    let entity_older = make_billing_period_entity(
        older_id,
        time::macros::date!(2025 - 01 - 01),
        time::macros::date!(2025 - 03 - 31),
    );

    let entity_older_clone = entity_older.clone();
    deps.billing_period_dao
        .expect_find_by_id()
        .with(eq(older_id), always())
        .times(1)
        .returning(move |_, _| Ok(Some(entity_older_clone.clone())));

    let all_entities: Arc<[dao::billing_period::BillingPeriodEntity]> =
        vec![entity_latest, entity_older].into();
    deps.billing_period_dao
        .expect_all_ordered_desc()
        .with(always())
        .times(1)
        .returning(move |_| Ok(all_entities.clone()));

    let service = deps.build_service();
    let result = service
        .delete_billing_period(older_id, context, None)
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ServiceError::NotLatestBillingPeriod(_)
    ));
}

#[tokio::test]
async fn test_delete_billing_period_not_found_returns_404() {
    let nonexistent_id = Uuid::new_v4();
    let context = Authentication::Full;

    let mut deps = MockDeps::new();
    setup_common_mocks(&mut deps);

    deps.permission_service
        .expect_check_permission()
        .with(
            eq(service::permission::HR_PRIVILEGE),
            eq(context.clone()),
        )
        .times(1)
        .returning(|_, _| Ok(()));

    deps.billing_period_dao
        .expect_find_by_id()
        .with(eq(nonexistent_id), always())
        .times(1)
        .returning(|_, _| Ok(None));

    let service = deps.build_service();
    let result = service
        .delete_billing_period(nonexistent_id, context, None)
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ServiceError::EntityNotFound(_)
    ));
}

#[tokio::test]
async fn test_delete_billing_period_forbidden_without_hr_privilege() {
    let id = Uuid::new_v4();
    let context = Authentication::Full;

    let mut deps = MockDeps::new();

    deps.permission_service
        .expect_check_permission()
        .with(
            eq(service::permission::HR_PRIVILEGE),
            eq(context.clone()),
        )
        .times(1)
        .returning(|_, _| Err(ServiceError::Forbidden));

    let service = deps.build_service();
    let result = service.delete_billing_period(id, context, None).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ServiceError::Forbidden));
}

#[tokio::test]
async fn test_delete_billing_period_cascades_sales_person_entries() {
    let latest_id = Uuid::new_v4();
    let context = Authentication::Full;

    let mut deps = MockDeps::new();
    setup_common_mocks(&mut deps);

    deps.permission_service
        .expect_check_permission()
        .with(
            eq(service::permission::HR_PRIVILEGE),
            eq(context.clone()),
        )
        .times(1)
        .returning(|_, _| Ok(()));

    let entity_latest = make_billing_period_entity(
        latest_id,
        time::macros::date!(2025 - 04 - 01),
        time::macros::date!(2025 - 06 - 30),
    );

    let entity_latest_clone = entity_latest.clone();
    deps.billing_period_dao
        .expect_find_by_id()
        .with(eq(latest_id), always())
        .times(1)
        .returning(move |_, _| Ok(Some(entity_latest_clone.clone())));

    let all_entities: Arc<[dao::billing_period::BillingPeriodEntity]> =
        vec![entity_latest].into();
    deps.billing_period_dao
        .expect_all_ordered_desc()
        .with(always())
        .times(1)
        .returning(move |_| Ok(all_entities.clone()));

    deps.permission_service
        .expect_current_user_id()
        .with(always())
        .times(1)
        .returning(|_| Ok(Some("test_user".into())));

    // Verify cascade: delete_by_billing_period_id is called BEFORE delete_by_id
    let mut seq = mockall::Sequence::new();

    deps.billing_period_sales_person_dao
        .expect_delete_by_billing_period_id()
        .with(eq(latest_id), eq("test_user"), always())
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_, _, _| Ok(()));

    deps.billing_period_dao
        .expect_delete_by_id()
        .with(eq(latest_id), eq("test_user"), always())
        .times(1)
        .in_sequence(&mut seq)
        .returning(|_, _, _| Ok(()));

    deps.transaction_dao
        .expect_commit()
        .with(always())
        .times(1)
        .returning(|_| Ok(()));

    let service = deps.build_service();
    let result = service
        .delete_billing_period(latest_id, context, None)
        .await;

    assert!(result.is_ok());
}
