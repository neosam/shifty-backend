//! Phase 4 / Plan 04-04 — service-level extra-hours tests covering:
//!
//! 1. flag-gated `create()` for the deprecated Vacation/SickLeave/UnpaidLeave
//!    categories (D-Phase4-09):
//!    - flag=off + Vacation -> Ok (legacy path)
//!    - flag=on  + Vacation -> Err(ExtraHoursCategoryDeprecated(Vacation))
//!    - flag=on  + ExtraWork -> Ok (ExtraWork is NOT in the gated set)
//! 2. `soft_delete_bulk()` (C-Phase4-04):
//!    - happy path: DAO is invoked with the verbatim id list + update_process
//!    - forbidden: permission gate sits BEFORE the DAO call (T-04-04-01)

use std::sync::Arc;

use mockall::predicate::{always, eq};
use time::{Date, Month, PrimitiveDateTime, Time};
use uuid::{uuid, Uuid};

use dao::extra_hours::MockExtraHoursDao;
use dao::{MockTransaction, MockTransactionDao};
use service::clock::MockClockService;
use service::custom_extra_hours::MockCustomExtraHoursService;
use service::cutover::CUTOVER_ADMIN_PRIVILEGE;
use service::extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursService};
use service::feature_flag::MockFeatureFlagService;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::sales_person::MockSalesPersonService;
use service::uuid_service::MockUuidService;
use service::{MockPermissionService, ServiceError};

use super::error_test::NoneTypeExt;
use crate::extra_hours::{ExtraHoursServiceDeps, ExtraHoursServiceImpl};

// ----------------------------------------------------------------------------
// Test harness — multi-mock dependency injection
// ----------------------------------------------------------------------------

pub struct ExtraHoursDependencies {
    pub extra_hours_dao: MockExtraHoursDao,
    pub permission_service: MockPermissionService,
    pub sales_person_service: MockSalesPersonService,
    pub custom_extra_hours_service: MockCustomExtraHoursService,
    pub feature_flag_service: MockFeatureFlagService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
    pub transaction_dao: MockTransactionDao,
}

impl ExtraHoursServiceDeps for ExtraHoursDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type ExtraHoursDao = MockExtraHoursDao;
    type PermissionService = MockPermissionService;
    type SalesPersonService = MockSalesPersonService;
    type CustomExtraHoursService = MockCustomExtraHoursService;
    type FeatureFlagService = MockFeatureFlagService;
    type ClockService = MockClockService;
    type UuidService = MockUuidService;
    type TransactionDao = MockTransactionDao;
}

impl ExtraHoursDependencies {
    pub fn build_service(self) -> ExtraHoursServiceImpl<ExtraHoursDependencies> {
        ExtraHoursServiceImpl {
            extra_hours_dao: self.extra_hours_dao.into(),
            permission_service: self.permission_service.into(),
            sales_person_service: self.sales_person_service.into(),
            custom_extra_hours_service: self.custom_extra_hours_service.into(),
            feature_flag_service: self.feature_flag_service.into(),
            clock_service: self.clock_service.into(),
            uuid_service: self.uuid_service.into(),
            transaction_dao: self.transaction_dao.into(),
        }
    }
}

fn build_default_transaction_dao() -> MockTransactionDao {
    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));
    transaction_dao
}

fn fixture_sales_person_id() -> Uuid {
    uuid!("AAAA0000-0000-0000-0000-000000000001")
}

fn fixture_new_id() -> Uuid {
    uuid!("BBBB0000-0000-0000-0000-000000000001")
}

fn fixture_new_version() -> Uuid {
    uuid!("CCCC0000-0000-0000-0000-000000000001")
}

fn fixture_now() -> PrimitiveDateTime {
    PrimitiveDateTime::new(
        Date::from_calendar_date(2026, Month::May, 3).unwrap(),
        Time::from_hms(12, 0, 0).unwrap(),
    )
}

fn fixture_extra_hours(category: ExtraHoursCategory) -> ExtraHours {
    ExtraHours {
        id: Uuid::nil(),
        sales_person_id: fixture_sales_person_id(),
        amount: 8.0,
        category,
        description: "test".into(),
        date_time: PrimitiveDateTime::new(
            Date::from_calendar_date(2026, Month::May, 4).unwrap(),
            Time::MIDNIGHT,
        ),
        created: None,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Build a base set of mocks where:
///  - permission HR is granted (so `hr_permission.or(...)` short-circuits)
///  - sales-person verification is permissive
///  - clock + uuid produce stable fixture values
fn build_dependencies_for_create() -> ExtraHoursDependencies {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Ok(()));
    permission_service
        .expect_check_permission()
        .returning(|_, _| Err(ServiceError::Forbidden));

    let mut sales_person_service = MockSalesPersonService::new();
    sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));

    let mut clock_service = MockClockService::new();
    clock_service.expect_date_time_now().returning(fixture_now);

    let mut uuid_service = MockUuidService::new();
    uuid_service
        .expect_new_uuid()
        .with(eq("extra_hours_service::create id"))
        .returning(|_| fixture_new_id());
    uuid_service
        .expect_new_uuid()
        .with(eq("extra_hours_service::create version"))
        .returning(|_| fixture_new_version());

    ExtraHoursDependencies {
        extra_hours_dao: MockExtraHoursDao::new(),
        permission_service,
        sales_person_service,
        custom_extra_hours_service: MockCustomExtraHoursService::new(),
        feature_flag_service: MockFeatureFlagService::new(),
        clock_service,
        uuid_service,
        transaction_dao: build_default_transaction_dao(),
    }
}

// ----------------------------------------------------------------------------
// Test 1 (D-Phase4-09): flag=off + Vacation -> Ok (legacy path remains active)
// ----------------------------------------------------------------------------

#[tokio::test]
async fn create_vacation_succeeds_when_flag_off() {
    let mut deps = build_dependencies_for_create();
    deps.feature_flag_service
        .expect_is_enabled()
        .with(eq("absence_range_source_active"), always(), always())
        .returning(|_, _, _| Ok(false));
    deps.extra_hours_dao
        .expect_create()
        .times(1)
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let entity = fixture_extra_hours(ExtraHoursCategory::Vacation);
    let result = service.create(&entity, ().auth(), None).await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    let created = result.unwrap();
    assert_eq!(created.id, fixture_new_id());
    assert_eq!(created.version, fixture_new_version());
}

// ----------------------------------------------------------------------------
// Test 2 (D-Phase4-09): flag=on + Vacation -> Err(ExtraHoursCategoryDeprecated)
// ----------------------------------------------------------------------------

#[tokio::test]
async fn create_vacation_returns_403_error_variant_when_flag_on() {
    let mut deps = build_dependencies_for_create();
    deps.feature_flag_service
        .expect_is_enabled()
        .with(eq("absence_range_source_active"), always(), always())
        .returning(|_, _, _| Ok(true));
    // CRITICAL: DAO must NOT be called when the flag-gate denies.
    deps.extra_hours_dao.expect_create().times(0);

    let service = deps.build_service();
    let entity = fixture_extra_hours(ExtraHoursCategory::Vacation);
    let result = service.create(&entity, ().auth(), None).await;
    assert!(
        matches!(
            result,
            Err(ServiceError::ExtraHoursCategoryDeprecated(
                ExtraHoursCategory::Vacation
            ))
        ),
        "expected ExtraHoursCategoryDeprecated(Vacation), got: {:?}",
        result
    );
}

// ----------------------------------------------------------------------------
// Test 3 (D-Phase4-09): flag=on + ExtraWork -> Ok (ExtraWork not in gated set)
// ----------------------------------------------------------------------------

#[tokio::test]
async fn create_extra_work_succeeds_when_flag_on() {
    let mut deps = build_dependencies_for_create();
    // Flag-check must be SKIPPED for non-deprecated categories — ExtraWork
    // is not in the {Vacation, SickLeave, UnpaidLeave} set, so the impl must
    // bypass `is_enabled`. Setting `times(0)` ensures we don't issue a
    // pointless flag-read on every ExtraWork POST.
    deps.feature_flag_service.expect_is_enabled().times(0);
    deps.extra_hours_dao
        .expect_create()
        .times(1)
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let entity = fixture_extra_hours(ExtraHoursCategory::ExtraWork);
    let result = service.create(&entity, ().auth(), None).await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

// ----------------------------------------------------------------------------
// Test 4 (C-Phase4-04 happy path): soft_delete_bulk forwards ids + tag verbatim
// ----------------------------------------------------------------------------

#[tokio::test]
async fn soft_delete_bulk_calls_dao_with_provided_ids_and_update_process() {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(eq(CUTOVER_ADMIN_PRIVILEGE), always())
        .returning(|_, _| Ok(()));

    let mut clock_service = MockClockService::new();
    clock_service.expect_date_time_now().returning(fixture_now);

    let mut uuid_service = MockUuidService::new();
    uuid_service
        .expect_new_uuid()
        .with(eq("extra_hours_service::soft_delete_bulk version"))
        .returning(|_| fixture_new_version());

    let id_a = uuid!("DDDD0000-0000-0000-0000-000000000001");
    let id_b = uuid!("DDDD0000-0000-0000-0000-000000000002");
    let id_c = uuid!("DDDD0000-0000-0000-0000-000000000003");

    let mut extra_hours_dao = MockExtraHoursDao::new();
    extra_hours_dao
        .expect_soft_delete_bulk()
        .withf(move |ids, deleted_at, update_process, version, _| {
            ids.len() == 3
                && ids.contains(&id_a)
                && ids.contains(&id_b)
                && ids.contains(&id_c)
                && *deleted_at == fixture_now()
                && update_process == "phase-4-cutover-migration"
                && *version == fixture_new_version()
        })
        .times(1)
        .returning(|_, _, _, _, _| Ok(()));

    let deps = ExtraHoursDependencies {
        extra_hours_dao,
        permission_service,
        sales_person_service: MockSalesPersonService::new(),
        custom_extra_hours_service: MockCustomExtraHoursService::new(),
        feature_flag_service: MockFeatureFlagService::new(),
        clock_service,
        uuid_service,
        transaction_dao: build_default_transaction_dao(),
    };

    let service = deps.build_service();
    let ids: Arc<[Uuid]> = Arc::from(vec![id_a, id_b, id_c]);
    let result = service
        .soft_delete_bulk(
            ids,
            "phase-4-cutover-migration",
            Authentication::Full,
            None,
        )
        .await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

// ----------------------------------------------------------------------------
// Test 5 (T-04-04-01): permission gate sits BEFORE the DAO call
// ----------------------------------------------------------------------------

#[tokio::test]
async fn soft_delete_bulk_forbidden_for_unprivileged_user() {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(eq(CUTOVER_ADMIN_PRIVILEGE), always())
        .returning(|_, _| Err(ServiceError::Forbidden));

    let mut extra_hours_dao = MockExtraHoursDao::new();
    // CRITICAL: this `.times(0)` is the proof that the permission gate sits
    // BEFORE the DAO call. If the impl ever calls the DAO before checking
    // permissions, this test fails with a mockall verification panic.
    extra_hours_dao.expect_soft_delete_bulk().times(0);

    // The Tx must NOT be opened either — the permission denial short-circuits
    // before `use_transaction`.
    let mut transaction_dao = MockTransactionDao::new();
    transaction_dao.expect_use_transaction().times(0);
    transaction_dao.expect_commit().times(0);

    let deps = ExtraHoursDependencies {
        extra_hours_dao,
        permission_service,
        sales_person_service: MockSalesPersonService::new(),
        custom_extra_hours_service: MockCustomExtraHoursService::new(),
        feature_flag_service: MockFeatureFlagService::new(),
        clock_service: MockClockService::new(),
        uuid_service: MockUuidService::new(),
        transaction_dao,
    };

    let service = deps.build_service();
    let ids: Arc<[Uuid]> = Arc::from(vec![uuid!("EEEE0000-0000-0000-0000-000000000001")]);
    let result = service
        .soft_delete_bulk(ids, "test-process", Authentication::Full, None)
        .await;
    assert!(
        matches!(result, Err(ServiceError::Forbidden)),
        "expected Forbidden, got: {:?}",
        result
    );
}
