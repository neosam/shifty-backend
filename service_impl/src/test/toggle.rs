use std::sync::Arc;

use crate::toggle::{ToggleServiceDeps, ToggleServiceImpl};
use dao::toggle::{MockToggleDao, ToggleEntity};
use dao::{MockTransaction, MockTransactionDao};
use mockall::predicate::{always, eq};
use service::permission::{Authentication, MockPermissionService};
use service::toggle::{Toggle, ToggleGroup, ToggleService, TOGGLE_ADMIN_PRIVILEGE};
use service::ServiceError;

// Dependencies for the Toggle service
pub struct ToggleServiceDependencies {
    pub toggle_dao: MockToggleDao,
    pub permission_service: MockPermissionService,
}

impl ToggleServiceDeps for ToggleServiceDependencies {
    type Context = ();
    type Transaction = MockTransaction;
    type ToggleDao = MockToggleDao;
    type PermissionService = MockPermissionService;
    type TransactionDao = MockTransactionDao;
}

impl ToggleServiceDependencies {
    pub fn build_service(self) -> ToggleServiceImpl<ToggleServiceDependencies> {
        let mut transaction_dao = MockTransactionDao::new();
        transaction_dao
            .expect_use_transaction()
            .returning(|_| Ok(MockTransaction));
        transaction_dao.expect_commit().returning(|_| Ok(()));

        ToggleServiceImpl {
            toggle_dao: self.toggle_dao.into(),
            permission_service: Arc::new(self.permission_service),
            transaction_dao: Arc::new(transaction_dao),
        }
    }
}

fn build_dependencies() -> ToggleServiceDependencies {
    ToggleServiceDependencies {
        toggle_dao: MockToggleDao::new(),
        permission_service: MockPermissionService::new(),
    }
}

fn default_toggle_entity() -> ToggleEntity {
    ToggleEntity {
        name: "test_toggle".to_string(),
        enabled: true,
        description: Some("Test toggle description".to_string()),
        value: None,
    }
}

fn default_toggle() -> Toggle {
    Toggle {
        name: "test_toggle".into(),
        enabled: true,
        description: Some("Test toggle description".into()),
        value: None,
    }
}

fn default_toggle_group() -> ToggleGroup {
    ToggleGroup {
        name: "test_group".into(),
        description: Some("Test group description".into()),
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

// Helper to mock authenticated user
fn mock_authenticated_permission_service() -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_current_user_id()
        .returning(|_| Ok(Some("test_user".into())));
    permission_service
}

// Helper to mock unauthenticated user
fn mock_unauthenticated_permission_service() -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_current_user_id()
        .returning(|_| Ok(None));
    permission_service
}

// Helper to mock toggle_admin privilege
fn mock_toggle_admin_permission_service() -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(eq(TOGGLE_ADMIN_PRIVILEGE), always())
        .returning(|_, _| Ok(()));
    permission_service
}

// Helper to mock missing toggle_admin privilege
fn mock_no_toggle_admin_permission_service() -> MockPermissionService {
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(eq(TOGGLE_ADMIN_PRIVILEGE), always())
        .returning(|_, _| Err(ServiceError::Forbidden));
    permission_service
}

// Tests for is_enabled

#[tokio::test]
async fn test_is_enabled_returns_true_for_enabled_toggle() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_authenticated_permission_service();
    deps.toggle_dao
        .expect_is_enabled()
        .with(eq("test_toggle"), always())
        .returning(|_, _| Ok(true));

    let service = deps.build_service();
    let result = service.is_enabled("test_toggle", ().auth(), None).await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_is_enabled_returns_false_for_disabled_toggle() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_authenticated_permission_service();
    deps.toggle_dao
        .expect_is_enabled()
        .with(eq("test_toggle"), always())
        .returning(|_, _| Ok(false));

    let service = deps.build_service();
    let result = service.is_enabled("test_toggle", ().auth(), None).await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_is_enabled_returns_false_for_nonexistent_toggle() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_authenticated_permission_service();
    deps.toggle_dao
        .expect_is_enabled()
        .with(eq("nonexistent"), always())
        .returning(|_, _| Ok(false));

    let service = deps.build_service();
    let result = service.is_enabled("nonexistent", ().auth(), None).await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_unauthenticated_user_cannot_read_toggles() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_unauthenticated_permission_service();

    let service = deps.build_service();
    let result = service.is_enabled("test_toggle", ().auth(), None).await;
    assert!(matches!(result, Err(ServiceError::Unauthorized)));
}

// Tests for get_all_toggles

#[tokio::test]
async fn test_get_all_toggles_returns_all_toggles() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_authenticated_permission_service();
    let entities: Arc<[ToggleEntity]> = vec![default_toggle_entity()].into();
    deps.toggle_dao
        .expect_get_all_toggles()
        .returning(move |_| Ok(entities.clone()));

    let service = deps.build_service();
    let result = service.get_all_toggles(().auth(), None).await;
    assert!(result.is_ok());
    let toggles = result.unwrap();
    assert_eq!(toggles.len(), 1);
    assert_eq!(toggles[0].name.as_ref(), "test_toggle");
}

// Tests for create_toggle

#[tokio::test]
async fn test_create_toggle_success() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_toggle_admin_permission_service();
    deps.toggle_dao
        .expect_create_toggle()
        .with(always(), eq("toggle-service"), always())
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let toggle = default_toggle();
    let result = service.create_toggle(&toggle, ().auth(), None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_toggle_requires_toggle_admin_privilege() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_no_toggle_admin_permission_service();

    let service = deps.build_service();
    let toggle = default_toggle();
    let result = service.create_toggle(&toggle, ().auth(), None).await;
    assert!(matches!(result, Err(ServiceError::Forbidden)));
}

// Tests for enable_toggle

#[tokio::test]
async fn test_enable_toggle_success() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_toggle_admin_permission_service();
    let mut entity = default_toggle_entity();
    entity.enabled = false;
    deps.toggle_dao
        .expect_get_toggle()
        .with(eq("test_toggle"), always())
        .returning(move |_, _| Ok(Some(entity.clone())));
    deps.toggle_dao
        .expect_update_toggle()
        .with(always(), eq("toggle-service"), always())
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let result = service.enable_toggle("test_toggle", ().auth(), None).await;
    assert!(result.is_ok());
}

// Tests for disable_toggle

#[tokio::test]
async fn test_disable_toggle_success() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_toggle_admin_permission_service();
    let entity = default_toggle_entity();
    deps.toggle_dao
        .expect_get_toggle()
        .with(eq("test_toggle"), always())
        .returning(move |_, _| Ok(Some(entity.clone())));
    deps.toggle_dao
        .expect_update_toggle()
        .with(always(), eq("toggle-service"), always())
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let result = service
        .disable_toggle("test_toggle", ().auth(), None)
        .await;
    assert!(result.is_ok());
}

// Tests for delete_toggle

#[tokio::test]
async fn test_delete_toggle_success() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_toggle_admin_permission_service();
    deps.toggle_dao
        .expect_delete_toggle()
        .with(eq("test_toggle"), eq("toggle-service"), always())
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let result = service.delete_toggle("test_toggle", ().auth(), None).await;
    assert!(result.is_ok());
}

// Tests for toggle groups

#[tokio::test]
async fn test_create_toggle_group_success() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_toggle_admin_permission_service();
    deps.toggle_dao
        .expect_create_toggle_group()
        .with(always(), eq("toggle-service"), always())
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let group = default_toggle_group();
    let result = service.create_toggle_group(&group, ().auth(), None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_toggle_group_success() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_toggle_admin_permission_service();
    deps.toggle_dao
        .expect_delete_toggle_group()
        .with(eq("test_group"), eq("toggle-service"), always())
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let result = service
        .delete_toggle_group("test_group", ().auth(), None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_toggle_to_group_success() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_toggle_admin_permission_service();
    deps.toggle_dao
        .expect_add_toggle_to_group()
        .with(
            eq("test_group"),
            eq("test_toggle"),
            eq("toggle-service"),
            always(),
        )
        .returning(|_, _, _, _| Ok(()));

    let service = deps.build_service();
    let result = service
        .add_toggle_to_group("test_group", "test_toggle", ().auth(), None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_remove_toggle_from_group_success() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_toggle_admin_permission_service();
    deps.toggle_dao
        .expect_remove_toggle_from_group()
        .with(
            eq("test_group"),
            eq("test_toggle"),
            eq("toggle-service"),
            always(),
        )
        .returning(|_, _, _, _| Ok(()));

    let service = deps.build_service();
    let result = service
        .remove_toggle_from_group("test_group", "test_toggle", ().auth(), None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_enable_group_enables_all_toggles_in_group() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_toggle_admin_permission_service();
    deps.toggle_dao
        .expect_enable_group()
        .with(eq("test_group"), eq("toggle-service"), always())
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let result = service.enable_group("test_group", ().auth(), None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_disable_group_disables_all_toggles_in_group() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_toggle_admin_permission_service();
    deps.toggle_dao
        .expect_disable_group()
        .with(eq("test_group"), eq("toggle-service"), always())
        .returning(|_, _, _| Ok(()));

    let service = deps.build_service();
    let result = service.disable_group("test_group", ().auth(), None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_non_admin_cannot_modify_toggles() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_no_toggle_admin_permission_service();

    let service = deps.build_service();
    let toggle = default_toggle();
    let result = service.create_toggle(&toggle, ().auth(), None).await;
    assert!(matches!(result, Err(ServiceError::Forbidden)));
}

// Tests for get_toggle_value / set_toggle_value roundtrip (HCFG-02)

#[tokio::test]
async fn test_toggle_value_roundtrip() {
    // Phase 1: Set a value (admin-gated) — should succeed
    {
        let mut deps = build_dependencies();
        deps.permission_service = mock_toggle_admin_permission_service();
        deps.toggle_dao
            .expect_set_toggle_value()
            .with(
                eq("holiday_auto_credit"),
                eq(Some("2026-01-01".to_string())),
                eq("toggle-service"),
                always(),
            )
            .returning(|_, _, _, _| Ok(()));

        let service = deps.build_service();
        let result = service
            .set_toggle_value(
                "holiday_auto_credit",
                Some("2026-01-01".to_string()),
                ().auth(),
                None,
            )
            .await;
        assert!(result.is_ok(), "set_toggle_value should succeed for admin");
    }

    // Phase 2: Read the value back (authenticated user) — should equal what was set
    {
        let mut deps = build_dependencies();
        deps.permission_service = mock_authenticated_permission_service();
        deps.toggle_dao
            .expect_get_toggle_value()
            .with(eq("holiday_auto_credit"), always())
            .returning(|_, _| Ok(Some("2026-01-01".to_string())));

        let service = deps.build_service();
        let result = service
            .get_toggle_value("holiday_auto_credit", ().auth(), None)
            .await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().as_deref(),
            Some("2026-01-01"),
            "get_toggle_value must return the previously set ISO date"
        );
    }

    // Phase 3: Clear the value (admin-gated, None) — should succeed
    {
        let mut deps = build_dependencies();
        deps.permission_service = mock_toggle_admin_permission_service();
        deps.toggle_dao
            .expect_set_toggle_value()
            .with(
                eq("holiday_auto_credit"),
                eq(None::<String>),
                eq("toggle-service"),
                always(),
            )
            .returning(|_, _, _, _| Ok(()));

        let service = deps.build_service();
        let result = service
            .set_toggle_value("holiday_auto_credit", None, ().auth(), None)
            .await;
        assert!(result.is_ok(), "clearing value (None) should succeed for admin");
    }

    // Phase 4: After clearing, get_toggle_value returns None (D-25-05 default off)
    {
        let mut deps = build_dependencies();
        deps.permission_service = mock_authenticated_permission_service();
        deps.toggle_dao
            .expect_get_toggle_value()
            .with(eq("holiday_auto_credit"), always())
            .returning(|_, _| Ok(None));

        let service = deps.build_service();
        let result = service
            .get_toggle_value("holiday_auto_credit", ().auth(), None)
            .await;
        assert!(result.is_ok());
        assert!(
            result.unwrap().is_none(),
            "get_toggle_value should return None after clearing (automation off)"
        );
    }

    // Phase 5: is_enabled is false after clearing (D-25-05 enabled mirrors value presence)
    {
        let mut deps = build_dependencies();
        deps.permission_service = mock_authenticated_permission_service();
        deps.toggle_dao
            .expect_is_enabled()
            .with(eq("holiday_auto_credit"), always())
            .returning(|_, _| Ok(false));

        let service = deps.build_service();
        let result = service
            .is_enabled("holiday_auto_credit", ().auth(), None)
            .await;
        assert!(result.is_ok());
        assert!(
            !result.unwrap(),
            "Toggle should be disabled after clearing value (D-25-05)"
        );
    }
}

#[tokio::test]
async fn test_set_toggle_value_requires_toggle_admin_privilege() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_no_toggle_admin_permission_service();

    let service = deps.build_service();
    let result = service
        .set_toggle_value(
            "holiday_auto_credit",
            Some("2026-01-01".to_string()),
            ().auth(),
            None,
        )
        .await;
    assert!(
        matches!(result, Err(ServiceError::Forbidden)),
        "set_toggle_value must be admin-gated"
    );
}

#[tokio::test]
async fn test_get_toggle_value_requires_authentication() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_unauthenticated_permission_service();

    let service = deps.build_service();
    let result = service
        .get_toggle_value("holiday_auto_credit", ().auth(), None)
        .await;
    assert!(
        matches!(result, Err(ServiceError::Unauthorized)),
        "get_toggle_value must require authentication"
    );
}

// ─── Gap-Closure Phase 51: Authentication::Full bypass ──────────────────────
//
// Regression-Guard für den Root-Fix: die vier Read-Ops (`is_enabled`,
// `get_all_toggles`, `get_toggle`, `get_toggle_value`) müssen mit
// `Authentication::Full` erfolgreich lesen, ohne den `current_user_id`-Guard
// auszulösen.
//
// **Live-Symptom vor Fix:** Chain C (`booking_information.rs`) und Chain D
// (`reporting.rs`, `shiftplan_report.rs`) riefen `get_toggle_value` intern mit
// `Authentication::Full` — der `PermissionService::current_user_id` liefert für
// `Full` `Ok(None)` → `Unauthorized` → `shortday_gate::read_active_from` fiel
// stumm auf `Ok(None)` zurück → `shortday_slot_clipping_active_from` wurde nie
// gelesen → volle Slot-Stunde statt geklippter 0,5h.

/// Für Full-Kontext DARF `current_user_id` gar nicht aufgerufen werden —
/// `MockPermissionService` ohne `expect_current_user_id` würde bei einem
/// Aufruf panicken. Wenn der Test grün ist, ist der Bypass korrekt verdrahtet.
fn mock_permission_service_that_must_not_be_called() -> MockPermissionService {
    // Kein `.expect_current_user_id(...)` — jeder Aufruf wäre ein Test-Fehler.
    MockPermissionService::new()
}

#[tokio::test]
async fn test_get_toggle_value_bypasses_user_id_check_for_full_auth() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_permission_service_that_must_not_be_called();
    deps.toggle_dao
        .expect_get_toggle_value()
        .with(eq("shortday_slot_clipping_active_from"), always())
        .returning(|_, _| Ok(Some("2026-06-28".to_string())));

    let service = deps.build_service();
    let result = service
        .get_toggle_value(
            "shortday_slot_clipping_active_from",
            Authentication::Full,
            None,
        )
        .await;
    assert!(
        result.is_ok(),
        "get_toggle_value(Authentication::Full) must NOT return Unauthorized (Phase 51 Gap-Closure)"
    );
    assert_eq!(
        result.unwrap().as_deref(),
        Some("2026-06-28"),
        "Full-Bypass darf den Toggle-Wert nicht verschlucken"
    );
}

#[tokio::test]
async fn test_is_enabled_bypasses_user_id_check_for_full_auth() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_permission_service_that_must_not_be_called();
    deps.toggle_dao
        .expect_is_enabled()
        .with(eq("shortday_slot_clipping_active_from"), always())
        .returning(|_, _| Ok(true));

    let service = deps.build_service();
    let result = service
        .is_enabled("shortday_slot_clipping_active_from", Authentication::Full, None)
        .await;
    assert!(
        matches!(result, Ok(true)),
        "is_enabled(Authentication::Full) must NOT return Unauthorized"
    );
}

#[tokio::test]
async fn test_get_toggle_bypasses_user_id_check_for_full_auth() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_permission_service_that_must_not_be_called();
    deps.toggle_dao
        .expect_get_toggle()
        .with(eq("test_toggle"), always())
        .returning(move |_, _| Ok(Some(default_toggle_entity())));

    let service = deps.build_service();
    let result = service
        .get_toggle("test_toggle", Authentication::Full, None)
        .await;
    assert!(
        result.is_ok(),
        "get_toggle(Authentication::Full) must NOT return Unauthorized"
    );
    assert!(result.unwrap().is_some());
}

#[tokio::test]
async fn test_get_all_toggles_bypasses_user_id_check_for_full_auth() {
    let mut deps = build_dependencies();
    deps.permission_service = mock_permission_service_that_must_not_be_called();
    let entities: Arc<[ToggleEntity]> = vec![default_toggle_entity()].into();
    deps.toggle_dao
        .expect_get_all_toggles()
        .returning(move |_| Ok(entities.clone()));

    let service = deps.build_service();
    let result = service.get_all_toggles(Authentication::Full, None).await;
    assert!(
        result.is_ok(),
        "get_all_toggles(Authentication::Full) must NOT return Unauthorized"
    );
    assert_eq!(result.unwrap().len(), 1);
}
