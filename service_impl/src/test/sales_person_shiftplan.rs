use crate::test::error_test::*;
use dao::{
    sales_person_shiftplan::MockSalesPersonShiftplanDao, MockTransaction, MockTransactionDao,
};
use service::{
    permission::MockPermissionService,
    sales_person::{MockSalesPersonService, SalesPerson},
    sales_person_shiftplan::SalesPersonShiftplanService,
};
use std::sync::Arc;
use uuid::{uuid, Uuid};

use crate::sales_person_shiftplan::{
    SalesPersonShiftplanServiceDeps, SalesPersonShiftplanServiceImpl,
};

fn default_sales_person_id() -> Uuid {
    uuid!("04215DFE-13C4-413C-8C66-77AC741BB5F0")
}

fn alternate_sales_person_id() -> Uuid {
    uuid!("04215DFE-13C4-413C-8C66-77AC741BB5F1")
}

fn third_sales_person_id() -> Uuid {
    uuid!("04215DFE-13C4-413C-8C66-77AC741BB5F2")
}

fn default_shiftplan_id() -> Uuid {
    uuid!("00000000-0000-4000-8000-000000000001")
}

fn alternate_shiftplan_id() -> Uuid {
    uuid!("00000000-0000-4000-8000-000000000002")
}

fn default_sales_person() -> SalesPerson {
    SalesPerson {
        id: default_sales_person_id(),
        name: "Max Mustermann".into(),
        background_color: "#FF0000".into(),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: Uuid::new_v4(),
    }
}

fn alternate_sales_person() -> SalesPerson {
    SalesPerson {
        id: alternate_sales_person_id(),
        name: "Erika Musterfrau".into(),
        background_color: "#00FF00".into(),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: Uuid::new_v4(),
    }
}

fn third_sales_person() -> SalesPerson {
    SalesPerson {
        id: third_sales_person_id(),
        name: "Hans Test".into(),
        background_color: "#0000FF".into(),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: Uuid::new_v4(),
    }
}

pub struct TestDependencies {
    pub sales_person_shiftplan_dao: MockSalesPersonShiftplanDao,
    pub sales_person_service: MockSalesPersonService,
    pub permission_service: MockPermissionService,
    pub transaction_dao: MockTransactionDao,
}

impl SalesPersonShiftplanServiceDeps for TestDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type SalesPersonShiftplanDao = MockSalesPersonShiftplanDao;
    type SalesPersonService = MockSalesPersonService;
    type PermissionService = MockPermissionService;
    type TransactionDao = MockTransactionDao;
}

impl TestDependencies {
    pub fn build_service(self) -> SalesPersonShiftplanServiceImpl<TestDependencies> {
        SalesPersonShiftplanServiceImpl {
            sales_person_shiftplan_dao: self.sales_person_shiftplan_dao.into(),
            sales_person_service: self.sales_person_service.into(),
            permission_service: self.permission_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

fn build_dependencies() -> TestDependencies {
    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_exists()
        .returning(|_, _, _| Ok(true));

    TestDependencies {
        sales_person_shiftplan_dao: MockSalesPersonShiftplanDao::new(),
        sales_person_service,
        permission_service,
        transaction_dao,
    }
}

// ===== Task 4.1: Setting, getting, and clearing assignments =====

#[tokio::test]
async fn test_get_shiftplans_for_sales_person() {
    let mut deps = build_dependencies();
    let plan_ids = vec![default_shiftplan_id(), alternate_shiftplan_id()];
    let plan_ids_clone = plan_ids.clone();
    deps.sales_person_shiftplan_dao
        .expect_get_by_sales_person()
        .returning(move |_, _| Ok(plan_ids_clone.clone()));

    let service = deps.build_service();
    let result = service
        .get_shiftplans_for_sales_person(default_sales_person_id(), ().auth(), None)
        .await;
    assert!(result.is_ok());
    let ids = result.unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&default_shiftplan_id()));
    assert!(ids.contains(&alternate_shiftplan_id()));
}

#[tokio::test]
async fn test_get_shiftplans_for_sales_person_empty() {
    let mut deps = build_dependencies();
    deps.sales_person_shiftplan_dao
        .expect_get_by_sales_person()
        .returning(|_, _| Ok(vec![]));

    let service = deps.build_service();
    let result = service
        .get_shiftplans_for_sales_person(default_sales_person_id(), ().auth(), None)
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn test_set_shiftplans_for_sales_person() {
    let mut deps = build_dependencies();
    deps.sales_person_shiftplan_dao
        .expect_set_for_sales_person()
        .returning(|_, _, _, _| Ok(()));

    let service = deps.build_service();
    let plan_ids = vec![default_shiftplan_id(), alternate_shiftplan_id()];
    let result = service
        .set_shiftplans_for_sales_person(default_sales_person_id(), &plan_ids, ().auth(), None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_clear_shiftplans_for_sales_person() {
    let mut deps = build_dependencies();
    deps.sales_person_shiftplan_dao
        .expect_set_for_sales_person()
        .returning(|_, _, _, _| Ok(()));

    let service = deps.build_service();
    let result = service
        .set_shiftplans_for_sales_person(default_sales_person_id(), &[], ().auth(), None)
        .await;
    assert!(result.is_ok());
}

// ===== Task 4.2: get_bookable_sales_persons with permissive logic =====

#[tokio::test]
async fn test_get_bookable_no_assignments_returns_all() {
    let mut deps = build_dependencies();

    let all_persons: Arc<[SalesPerson]> = Arc::new([
        default_sales_person(),
        alternate_sales_person(),
        third_sales_person(),
    ]);
    let all_persons_clone = all_persons.clone();
    deps.sales_person_service
        .expect_get_all()
        .returning(move |_, _| Ok(all_persons_clone.clone()));

    // No one has any assignments
    deps.sales_person_shiftplan_dao
        .expect_has_any_assignment()
        .returning(|_, _| Ok(false));

    let service = deps.build_service();
    let result = service
        .get_bookable_sales_persons(default_shiftplan_id(), ().auth(), None)
        .await;
    assert!(result.is_ok());
    let bookable = result.unwrap();
    assert_eq!(bookable.len(), 3);
}

#[tokio::test]
async fn test_get_bookable_mixed_assignments() {
    let mut deps = build_dependencies();

    let all_persons: Arc<[SalesPerson]> = Arc::new([
        default_sales_person(),    // no assignments -> eligible everywhere
        alternate_sales_person(),  // assigned to default_shiftplan -> eligible
        third_sales_person(),      // assigned to alternate_shiftplan -> NOT eligible
    ]);
    let all_persons_clone = all_persons.clone();
    deps.sales_person_service
        .expect_get_all()
        .returning(move |_, _| Ok(all_persons_clone.clone()));

    let sp_a = default_sales_person_id();
    let sp_b = alternate_sales_person_id();
    let sp_c = third_sales_person_id();

    deps.sales_person_shiftplan_dao
        .expect_has_any_assignment()
        .returning(move |id, _| {
            Ok(id == sp_b || id == sp_c)
        });

    let plan_id = default_shiftplan_id();
    deps.sales_person_shiftplan_dao
        .expect_is_assigned()
        .returning(move |sp_id, shiftplan_id, _| {
            Ok(sp_id == sp_b && shiftplan_id == plan_id)
        });

    let service = deps.build_service();
    let result = service
        .get_bookable_sales_persons(default_shiftplan_id(), ().auth(), None)
        .await;
    assert!(result.is_ok());
    let bookable = result.unwrap();
    assert_eq!(bookable.len(), 2);
    assert!(bookable.iter().any(|sp| sp.id == sp_a));
    assert!(bookable.iter().any(|sp| sp.id == sp_b));
    assert!(!bookable.iter().any(|sp| sp.id == sp_c));
}

// ===== Task 4.3: is_eligible tests =====

#[tokio::test]
async fn test_is_eligible_no_assignments() {
    let mut deps = build_dependencies();
    deps.sales_person_shiftplan_dao
        .expect_has_any_assignment()
        .returning(|_, _| Ok(false));

    let service = deps.build_service();
    let result = service
        .is_eligible(default_sales_person_id(), default_shiftplan_id(), None)
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_is_eligible_assigned_to_plan() {
    let mut deps = build_dependencies();
    deps.sales_person_shiftplan_dao
        .expect_has_any_assignment()
        .returning(|_, _| Ok(true));
    deps.sales_person_shiftplan_dao
        .expect_is_assigned()
        .returning(|_, _, _| Ok(true));

    let service = deps.build_service();
    let result = service
        .is_eligible(default_sales_person_id(), default_shiftplan_id(), None)
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_is_eligible_assigned_to_other_plan() {
    let mut deps = build_dependencies();
    deps.sales_person_shiftplan_dao
        .expect_has_any_assignment()
        .returning(|_, _| Ok(true));
    deps.sales_person_shiftplan_dao
        .expect_is_assigned()
        .returning(|_, _, _| Ok(false));

    let service = deps.build_service();
    let result = service
        .is_eligible(default_sales_person_id(), default_shiftplan_id(), None)
        .await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

// ===== Task 4.5: Permission checks =====

#[tokio::test]
async fn test_set_shiftplans_forbidden() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Err(service::ServiceError::Forbidden));

    let service = deps.build_service();
    let result = service
        .set_shiftplans_for_sales_person(
            default_sales_person_id(),
            &[default_shiftplan_id()],
            ().auth(),
            None,
        )
        .await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_get_shiftplans_forbidden() {
    let mut deps = build_dependencies();
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .returning(|_, _| Err(service::ServiceError::Forbidden));

    let service = deps.build_service();
    let result = service
        .get_shiftplans_for_sales_person(default_sales_person_id(), ().auth(), None)
        .await;
    test_forbidden(&result);
}

// ===== Fix verification: sales person existence check =====

#[tokio::test]
async fn test_set_shiftplans_sales_person_not_found() {
    let mut deps = build_dependencies();
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_exists()
        .returning(|_, _, _| Ok(false));

    let service = deps.build_service();
    let result = service
        .set_shiftplans_for_sales_person(
            default_sales_person_id(),
            &[default_shiftplan_id()],
            ().auth(),
            None,
        )
        .await;
    test_not_found(&result, &default_sales_person_id());
}

// ===== Fix verification: inactive persons excluded from bookable =====

#[tokio::test]
async fn test_get_bookable_excludes_inactive() {
    let mut deps = build_dependencies();

    let mut inactive_person = alternate_sales_person();
    inactive_person.inactive = true;

    let all_persons: Arc<[SalesPerson]> = Arc::new([
        default_sales_person(),   // active, no assignments
        inactive_person,          // inactive, should be excluded
    ]);
    let all_persons_clone = all_persons.clone();
    deps.sales_person_service
        .expect_get_all()
        .returning(move |_, _| Ok(all_persons_clone.clone()));

    deps.sales_person_shiftplan_dao
        .expect_has_any_assignment()
        .returning(|_, _| Ok(false));

    let service = deps.build_service();
    let result = service
        .get_bookable_sales_persons(default_shiftplan_id(), ().auth(), None)
        .await;
    assert!(result.is_ok());
    let bookable = result.unwrap();
    assert_eq!(bookable.len(), 1);
    assert_eq!(bookable[0].id, default_sales_person_id());
}
