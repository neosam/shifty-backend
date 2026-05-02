use std::sync::Arc;

use crate::feature_flag::{FeatureFlagServiceDeps, FeatureFlagServiceImpl};
use dao::feature_flag::MockFeatureFlagDao;
use dao::{MockTransaction, MockTransactionDao};
use mockall::predicate::{always, eq};
use service::feature_flag::{FeatureFlagService, FEATURE_FLAG_ADMIN_PRIVILEGE};
use service::permission::{Authentication, MockPermissionService};
use service::ServiceError;

pub struct FeatureFlagServiceDependencies {
    pub feature_flag_dao: MockFeatureFlagDao,
    pub permission_service: MockPermissionService,
}

impl FeatureFlagServiceDeps for FeatureFlagServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type FeatureFlagDao = MockFeatureFlagDao;
    type PermissionService = MockPermissionService;
    type TransactionDao = MockTransactionDao;
}

impl FeatureFlagServiceDependencies {
    pub fn build_service(self) -> FeatureFlagServiceImpl<FeatureFlagServiceDependencies> {
        let mut transaction_dao = MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        FeatureFlagServiceImpl {
            feature_flag_dao: self.feature_flag_dao.into(),
            permission_service: Arc::new(self.permission_service),
            transaction_dao: Arc::new(transaction_dao),
        }
    }
}

fn build_dependencies() -> FeatureFlagServiceDependencies {
    FeatureFlagServiceDependencies {
        feature_flag_dao: MockFeatureFlagDao::new(),
        permission_service: MockPermissionService::new(),
    }
}

trait NoneTypeExt {
    fn auth(&self) -> Authentication<()>;
}
impl NoneTypeExt for () {
    fn auth(&self) -> Authentication<()> {
        Authentication::Context(())
    }
}

fn mock_authenticated_permission_service() -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_current_user_id()
        .returning(|_| Ok(Some("test_user".into())));
    permission_service
}

fn mock_unauthenticated_permission_service() -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_current_user_id()
        .returning(|_| Ok(None));
    permission_service
}

fn mock_feature_flag_admin_permission_service() -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(eq(FEATURE_FLAG_ADMIN_PRIVILEGE), always())
        .returning(|_, _| Ok(()));
    permission_service
}

fn mock_no_feature_flag_admin_permission_service() -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(eq(FEATURE_FLAG_ADMIN_PRIVILEGE), always())
        .returning(|_, _| Err(ServiceError::Forbidden));
    permission_service
}

// is_enabled tests --------------------------------------------------------

#[tokio::test]
async fn test_is_enabled_returns_dao_value() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_authenticated_permission_service();
    deps.feature_flag_dao
        .expect_is_enabled()
        .with(eq("test_key"), always())
        .returning(|_, _| Ok(true));

    let service = deps.build_service();
    let result = service.is_enabled("test_key", ().auth(), None).await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_is_enabled_returns_false_for_unknown_key() {
    // DAO already returns false fail-safe for unknown keys (Task 3.1).
    // The service must propagate that without wrapping it as an error.
    let mut deps = build_dependencies();
    deps.permission_service = mock_authenticated_permission_service();
    deps.feature_flag_dao
        .expect_is_enabled()
        .with(eq("nonexistent"), always())
        .returning(|_, _| Ok(false));

    let service = deps.build_service();
    let result = service.is_enabled("nonexistent", ().auth(), None).await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_is_enabled_unauthenticated_rejected() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_unauthenticated_permission_service();
    // DAO must NOT be called when unauthenticated.
    deps.feature_flag_dao.expect_is_enabled().times(0);

    let service = deps.build_service();
    let result = service
        .is_enabled("absence_range_source_active", ().auth(), None)
        .await;
    assert!(matches!(result, Err(ServiceError::Unauthorized)));
}

#[tokio::test]
async fn test_is_enabled_authentication_full_bypasses_user_check() {
    // Plan 02-04: ReportingService ruft is_enabled mit Authentication::Full
    // (Service-zu-Service-Aufruf, Backend-internal trust). Das darf NICHT
    // mit Unauthorized scheitern, auch wenn current_user_id None waere.
    let mut deps = build_dependencies();
    // Permission service muss NICHT konsultiert werden bei Authentication::Full.
    // Wenn die Implementation current_user_id() doch ruft, ist das ein Bug.
    deps.permission_service
        .expect_current_user_id()
        .times(0);
    deps.feature_flag_dao
        .expect_is_enabled()
        .with(eq("absence_range_source_active"), always())
        .returning(|_, _| Ok(true));

    let service = deps.build_service();
    let result = service
        .is_enabled("absence_range_source_active", Authentication::Full, None)
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap(), "Authentication::Full muss DAO-Wert durchreichen");
}

// set tests ---------------------------------------------------------------

#[tokio::test]
async fn test_set_requires_admin_permission() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_feature_flag_admin_permission_service();
    deps.feature_flag_dao
        .expect_set()
        .withf(|key, value, process, _| {
            key == "absence_range_source_active"
                && *value == true
                && process == "feature-flag-service"
        })
        .returning(|_, _, _, _| Ok(()));

    let service = deps.build_service();
    let result = service
        .set("absence_range_source_active", true, ().auth(), None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_forbidden_for_non_admin() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_no_feature_flag_admin_permission_service();
    // DAO must NOT be called when permission check fails.
    deps.feature_flag_dao.expect_set().times(0);

    let service = deps.build_service();
    let result = service
        .set("absence_range_source_active", true, ().auth(), None)
        .await;
    assert!(matches!(result, Err(ServiceError::Forbidden)));
}
