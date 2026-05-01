//! ExtraHoursService tests covering two distinct concerns:
//!
//! 1. Pre-Phase-4 logical_id update path (commit fe744df):
//!    - update soft-deletes the old physical row + inserts a new row carrying
//!      the same logical_id
//!    - permission OR-flow (HR or self), version conflict, sales_person_id
//!      change rejection, unknown/soft-deleted entries return NotFound
//!
//! 2. Phase 4 / Plan 04-04 — service-level flag-gate + soft_delete_bulk:
//!    - flag-gated `create()` for the deprecated Vacation/SickLeave/UnpaidLeave
//!      categories (D-Phase4-09):
//!      - flag=off + Vacation -> Ok (legacy path)
//!      - flag=on  + Vacation -> Err(ExtraHoursCategoryDeprecated(Vacation))
//!      - flag=on  + ExtraWork -> Ok (ExtraWork is NOT in the gated set)
//!    - `soft_delete_bulk()` (C-Phase4-04):
//!      - happy path: DAO is invoked with the verbatim id list + update_process
//!      - forbidden: permission gate sits BEFORE the DAO call (T-04-04-01)

use std::sync::Arc;

use dao::extra_hours::ExtraHoursCategoryEntity;
use dao::extra_hours::ExtraHoursEntity;
use dao::extra_hours::MockExtraHoursDao;
use dao::DaoError;
use dao::MockTransaction;
use dao::MockTransactionDao;
use mockall::predicate::always;
use mockall::predicate::eq;
use service::clock::MockClockService;
use service::custom_extra_hours::MockCustomExtraHoursService;
use service::cutover::CUTOVER_ADMIN_PRIVILEGE;
use service::extra_hours::ExtraHours;
use service::extra_hours::ExtraHoursCategory;
use service::extra_hours::ExtraHoursService;
use service::feature_flag::MockFeatureFlagService;
use service::permission::Authentication;
use service::permission::HR_PRIVILEGE;
use service::sales_person::MockSalesPersonService;
use service::uuid_service::MockUuidService;
use service::MockPermissionService;
use service::ServiceError;
use service::ValidationFailureItem;
use time::macros::datetime;
use time::{Date, Month, PrimitiveDateTime, Time};
use uuid::uuid;
use uuid::Uuid;

use crate::extra_hours::ExtraHoursServiceDeps;
use crate::extra_hours::ExtraHoursServiceImpl;
use crate::test::error_test::test_conflicts;
use crate::test::error_test::test_forbidden;
use crate::test::error_test::test_not_found;
use crate::test::error_test::test_validation_error;
use crate::test::error_test::NoneTypeExt;

// ----------------------------------------------------------------------------
// Shared fixtures (used by both sets of tests)
// ----------------------------------------------------------------------------

pub fn default_logical_id() -> Uuid {
    uuid!("AA000000-0000-0000-0000-000000000001")
}
pub fn default_physical_id() -> Uuid {
    // For the first version of the entry, physical id == logical id.
    default_logical_id()
}
pub fn alternate_physical_id() -> Uuid {
    uuid!("BB000000-0000-0000-0000-000000000002")
}
pub fn default_sales_person_id() -> Uuid {
    uuid!("CC000000-0000-0000-0000-000000000003")
}
pub fn other_sales_person_id() -> Uuid {
    uuid!("CC000000-0000-0000-0000-000000000004")
}
pub fn default_version() -> Uuid {
    uuid!("DD000000-0000-0000-0000-000000000005")
}
pub fn alternate_version() -> Uuid {
    uuid!("DD000000-0000-0000-0000-000000000006")
}
pub fn unknown_logical_id() -> Uuid {
    uuid!("EE000000-0000-0000-0000-000000000007")
}

pub fn default_active_entity() -> ExtraHoursEntity {
    ExtraHoursEntity {
        id: default_physical_id(),
        logical_id: default_logical_id(),
        sales_person_id: default_sales_person_id(),
        amount: 4.0,
        category: ExtraHoursCategoryEntity::ExtraWork,
        description: "old description".into(),
        date_time: datetime!(2026-04-12 8:00:00),
        created: datetime!(2026-04-12 9:00:00),
        deleted: None,
        version: default_version(),
    }
}

pub fn default_update_request() -> ExtraHours {
    ExtraHours {
        id: default_logical_id(),
        sales_person_id: default_sales_person_id(),
        amount: 5.5,
        category: ExtraHoursCategory::ExtraWork,
        description: "corrected description".into(),
        date_time: datetime!(2026-04-12 8:00:00),
        created: None,
        deleted: None,
        version: default_version(),
    }
}

// ----------------------------------------------------------------------------
// Multi-mock test harness — single struct serving both test families
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

// ----------------------------------------------------------------------------
// Helper for the logical_id update tests (Side #2 — pre-Phase-4)
// ----------------------------------------------------------------------------

fn build_dependencies() -> ExtraHoursDependencies {
    let extra_hours_dao = MockExtraHoursDao::new();
    let mut permission_service = MockPermissionService::new();
    let mut sales_person_service = MockSalesPersonService::new();
    let custom_extra_hours_service = MockCustomExtraHoursService::new();
    let feature_flag_service = MockFeatureFlagService::new();
    let mut clock_service = MockClockService::new();
    let uuid_service = MockUuidService::new();
    let mut transaction_dao = MockTransactionDao::new();

    permission_service
        .expect_check_permission()
        .returning(|_, _| Ok(()));
    sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Ok(()));
    clock_service
        .expect_date_time_now()
        .returning(|| datetime!(2026-04-28 12:00:00));
    transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    transaction_dao.expect_commit().returning(|_| Ok(()));

    ExtraHoursDependencies {
        extra_hours_dao,
        permission_service,
        sales_person_service,
        custom_extra_hours_service,
        feature_flag_service,
        clock_service,
        uuid_service,
        transaction_dao,
    }
}

// ----------------------------------------------------------------------------
// Helpers for the Phase-4 flag-gate tests (Side #1 — Plan 04-04)
// ----------------------------------------------------------------------------

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

// ============================================================================
// Tests — logical_id update path (commit fe744df)
// ============================================================================

#[tokio::test]
async fn test_update_success_soft_deletes_old_inserts_new() {
    let mut deps = build_dependencies();
    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .with(eq(default_logical_id()), always())
        .returning(|_, _| Ok(Some(default_active_entity())));
    deps.extra_hours_dao
        .expect_update()
        .withf(|entity: &ExtraHoursEntity, _process: &str, _tx| {
            entity.id == default_physical_id() && entity.deleted.is_some()
        })
        .returning(|_, _, _| Ok(()));
    deps.extra_hours_dao
        .expect_create()
        .withf(|entity: &ExtraHoursEntity, _process: &str, _tx| {
            entity.id == alternate_physical_id()
                && entity.logical_id == default_logical_id()
                && entity.amount == 5.5
                && entity.version == alternate_version()
                && entity.deleted.is_none()
                && entity.created == datetime!(2026-04-28 12:00:00)
        })
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("extra_hours_service::update::id"))
        .returning(|_| alternate_physical_id());
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("extra_hours_service::update::version"))
        .returning(|_| alternate_version());
    let service = deps.build_service();

    let result = service
        .update(&default_update_request(), Authentication::Full, None)
        .await;

    let updated = result.expect("update should succeed");
    assert_eq!(updated.id, default_logical_id());
    assert_eq!(updated.version, alternate_version());
    assert_eq!(updated.amount, 5.5);
    assert_eq!(
        updated.created,
        Some(datetime!(2026-04-28 12:00:00))
    );
}

#[tokio::test]
async fn test_update_insert_failure_propagates_error() {
    // Single transaction guarantee: if the insert fails, the soft-delete
    // is rolled back (verified by the Service returning Err and not
    // committing). With mockall we observe this via the Err propagation;
    // commit() is not expected to be called.
    let mut deps = build_dependencies();
    // No commit expected on insert failure.
    deps.transaction_dao.checkpoint();
    deps.transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(MockTransaction));
    // Deliberately do NOT register an expectation for commit — if the service
    // tries to commit we want the test to fail.

    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    deps.extra_hours_dao
        .expect_update()
        .returning(|_, _, _| Ok(()));
    deps.extra_hours_dao
        .expect_create()
        .returning(|_, _, _| Err(DaoError::EnumValueNotFound("simulated".into())));
    deps.uuid_service
        .expect_new_uuid()
        .returning(|_| alternate_physical_id());
    let service = deps.build_service();

    let result = service
        .update(&default_update_request(), Authentication::Full, None)
        .await;

    assert!(
        result.is_err(),
        "Insert failure should propagate as Err and prevent commit"
    );
}

#[tokio::test]
async fn test_update_stale_version_returns_conflict() {
    let mut deps = build_dependencies();
    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    let service = deps.build_service();

    let stale_request = ExtraHours {
        version: alternate_version(),
        ..default_update_request()
    };

    let result = service.update(&stale_request, Authentication::Full, None).await;

    test_conflicts(
        &result,
        &default_logical_id(),
        &alternate_version(),
        &default_version(),
    );
}

#[tokio::test]
async fn test_update_changing_sales_person_id_is_rejected() {
    let mut deps = build_dependencies();
    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    let service = deps.build_service();

    let bad_request = ExtraHours {
        sales_person_id: other_sales_person_id(),
        ..default_update_request()
    };

    let result = service.update(&bad_request, Authentication::Full, None).await;

    test_validation_error(
        &result,
        &ValidationFailureItem::ModificationNotAllowed("sales_person_id".into()),
        1,
    );
}

#[tokio::test]
async fn test_update_self_can_update_own_entry() {
    let mut deps = build_dependencies();
    // HR fails, sales person passes — verifies the OR-permission flow.
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(service::ServiceError::Forbidden));
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .with(eq(default_sales_person_id()), always(), always())
        .returning(|_, _, _| Ok(()));
    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    deps.extra_hours_dao
        .expect_update()
        .returning(|_, _, _| Ok(()));
    deps.extra_hours_dao
        .expect_create()
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("extra_hours_service::update::id"))
        .returning(|_| alternate_physical_id());
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("extra_hours_service::update::version"))
        .returning(|_| alternate_version());
    let service = deps.build_service();

    let result = service
        .update(&default_update_request(), Authentication::Full, None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_other_sales_person_without_hr_is_forbidden() {
    let mut deps = build_dependencies();
    // HR fails, sales-person verification also fails. The active row is still
    // loaded first because the service needs its sales_person_id to do the
    // fine-grained permission check.
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Err(service::ServiceError::Forbidden));
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(service::ServiceError::Forbidden));
    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    let service = deps.build_service();

    let result = service
        .update(&default_update_request(), Authentication::Full, None)
        .await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_update_hr_can_update_any_entry() {
    let mut deps = build_dependencies();
    // sales_person verification fails, but HR passes.
    deps.permission_service.checkpoint();
    deps.permission_service
        .expect_check_permission()
        .with(eq(HR_PRIVILEGE), always())
        .returning(|_, _| Ok(()));
    deps.sales_person_service.checkpoint();
    deps.sales_person_service
        .expect_verify_user_is_sales_person()
        .returning(|_, _, _| Err(service::ServiceError::Forbidden));
    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    deps.extra_hours_dao
        .expect_update()
        .returning(|_, _, _| Ok(()));
    deps.extra_hours_dao
        .expect_create()
        .returning(|_, _, _| Ok(()));
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("extra_hours_service::update::id"))
        .returning(|_| alternate_physical_id());
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("extra_hours_service::update::version"))
        .returning(|_| alternate_version());
    let service = deps.build_service();

    let result = service
        .update(&default_update_request(), Authentication::Full, None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_unknown_logical_id_returns_not_found() {
    let mut deps = build_dependencies();
    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(None));
    let service = deps.build_service();

    let request = ExtraHours {
        id: unknown_logical_id(),
        ..default_update_request()
    };
    let result = service.update(&request, Authentication::Full, None).await;
    test_not_found(&result, &unknown_logical_id());
}

#[tokio::test]
async fn test_update_soft_deleted_entry_returns_not_found() {
    // When all rows for a logical_id are deleted, find_by_logical_id returns
    // None (because of the WHERE deleted IS NULL filter). Same shape as unknown.
    let mut deps = build_dependencies();
    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(None));
    let service = deps.build_service();

    let result = service
        .update(&default_update_request(), Authentication::Full, None)
        .await;
    test_not_found(&result, &default_logical_id());
}

#[tokio::test]
async fn test_update_persists_editable_fields_to_new_row() {
    let captured: Arc<std::sync::Mutex<Option<ExtraHoursEntity>>> =
        Arc::new(std::sync::Mutex::new(None));
    let captured_clone = captured.clone();

    let mut deps = build_dependencies();
    deps.extra_hours_dao
        .expect_find_by_logical_id()
        .returning(|_, _| Ok(Some(default_active_entity())));
    deps.extra_hours_dao
        .expect_update()
        .returning(|_, _, _| Ok(()));
    deps.extra_hours_dao
        .expect_create()
        .returning(move |entity: &ExtraHoursEntity, _process, _tx| {
            *captured_clone.lock().unwrap() = Some(entity.clone());
            Ok(())
        });
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("extra_hours_service::update::id"))
        .returning(|_| alternate_physical_id());
    deps.uuid_service
        .expect_new_uuid()
        .with(eq("extra_hours_service::update::version"))
        .returning(|_| alternate_version());
    let service = deps.build_service();

    let request = ExtraHours {
        amount: 7.25,
        description: "edited description".into(),
        date_time: datetime!(2026-04-13 10:30:00),
        category: ExtraHoursCategory::SickLeave,
        ..default_update_request()
    };
    let _ = service.update(&request, Authentication::Full, None).await;

    let new_row = captured
        .lock()
        .unwrap()
        .clone()
        .expect("create should be called with the new row");
    assert_eq!(new_row.amount, 7.25);
    assert_eq!(new_row.description.as_ref(), "edited description");
    assert_eq!(new_row.date_time, datetime!(2026-04-13 10:30:00));
    assert_eq!(new_row.category, ExtraHoursCategoryEntity::SickLeave);
    assert_eq!(new_row.logical_id, default_logical_id());
    assert_eq!(new_row.id, alternate_physical_id());
    assert!(new_row.deleted.is_none());
}

// ============================================================================
// Tests — Phase 4 / Plan 04-04 (flag-gate + soft_delete_bulk)
// ============================================================================

// Test 1 (D-Phase4-09): flag=off + Vacation -> Ok (legacy path remains active)
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

// Test 2 (D-Phase4-09): flag=on + Vacation -> Err(ExtraHoursCategoryDeprecated)
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

// Test 3 (D-Phase4-09): flag=on + ExtraWork -> Ok (ExtraWork not in gated set)
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

// Test 4 (C-Phase4-04 happy path): soft_delete_bulk forwards ids + tag verbatim
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

// Test 5 (T-04-04-01): permission gate sits BEFORE the DAO call
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
