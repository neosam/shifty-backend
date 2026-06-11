//! Phase 8 Plan 08-07 Gap-Closure (Task 2) — Feature-Flag REST-Endpoint
//! End-to-End-Tests.
//!
//! Phase 8.6 re-key (D-06): Tests that previously used "absence_range_source_active"
//! (which was removed in Phase 8.6) now use "totally_unknown_flag_xyz" to test the
//! generic fail-safe-false mechanism.
//!
//! Coverage:
//! 1. `is_enabled` über Service-Layer mit `Authentication::Full` —
//!    unbekannte Keys → fail-safe `false`.
//! 2. `is_enabled` mit `Authentication::Context(None)` → `Unauthorized`.
//! 3. REST-Layer (`GET /feature-flag/{key}`) via tower::ServiceExt::oneshot,
//!    damit der HTTP-Pfad (URL + Handler + DTO + JSON-Serialisierung)
//!    end-to-end durchgefahren wird.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Extension;
use http_body_util::BodyExt;
use rest::feature_flag::generate_route;
use rest::{Context as RestContext, RestStateDef};
use rest_types::FeatureFlagTO;
use service::feature_flag::FeatureFlagService;
use service::permission::Authentication;
use service::ServiceError;
use tower::ServiceExt;

use crate::integration_test::TestSetup;

#[tokio::test]
async fn service_layer_unknown_flag_returns_false_failsafe() {
    let test_setup = TestSetup::new().await;
    let svc = test_setup.rest_state.feature_flag_service();
    let value = svc
        .is_enabled("totally_unknown_flag_xyz", Authentication::Full, None)
        .await
        .unwrap();
    assert!(!value, "unknown flag must return false (fail-safe)");
}

#[tokio::test]
async fn service_layer_unauthenticated_context_is_rejected() {
    let test_setup = TestSetup::new().await;
    let svc = test_setup.rest_state.feature_flag_service();
    let res = svc
        .is_enabled(
            "totally_unknown_flag_xyz",
            Authentication::Context(None),
            None,
        )
        .await;
    assert!(
        matches!(res, Err(ServiceError::Unauthorized)),
        "unauth context must return ServiceError::Unauthorized, got: {:?}",
        res
    );
}

/// REST-Pfad: unbekannter Key, authentifizierter User → 200 mit `enabled: false` (fail-safe).
#[tokio::test]
async fn rest_get_known_flag_returns_200_with_body() {
    let test_setup = TestSetup::new().await;
    crate::create_admin_user(test_setup.pool.clone(), "ff_test_user").await;

    let router = axum::Router::new()
        .nest("/feature-flag", generate_route::<crate::RestStateImpl>())
        .with_state(test_setup.rest_state.clone())
        .layer(Extension(
            Some(Arc::<str>::from("ff_test_user")) as RestContext,
        ));

    let req = Request::builder()
        .method("GET")
        .uri("/feature-flag/totally_unknown_flag_xyz")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let to: FeatureFlagTO = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(to.key, "totally_unknown_flag_xyz");
    assert!(!to.enabled, "unknown flag fail-safe to disabled");
}

/// REST-Pfad: unbekannter Key, authentifizierter User → 200 mit `enabled: false`.
#[tokio::test]
async fn rest_get_unknown_flag_returns_200_with_disabled_failsafe() {
    let test_setup = TestSetup::new().await;
    crate::create_admin_user(test_setup.pool.clone(), "ff_test_user").await;

    let router = axum::Router::new()
        .nest("/feature-flag", generate_route::<crate::RestStateImpl>())
        .with_state(test_setup.rest_state.clone())
        .layer(Extension(
            Some(Arc::<str>::from("ff_test_user")) as RestContext,
        ));

    let req = Request::builder()
        .method("GET")
        .uri("/feature-flag/some_completely_unknown_flag")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let to: FeatureFlagTO = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(to.key, "some_completely_unknown_flag");
    assert!(!to.enabled, "unknown key fail-safe to disabled");
    assert_eq!(to.description, None);
}

/// REST-Pfad: unauthenticated context → 401.
/// Der Handler nimmt `Extension<Context>` (Context = Option<Arc<str>>).
/// Ein leerer Context (None) führt im Service zu Unauthorized → 401.
#[tokio::test]
async fn rest_get_without_user_returns_401() {
    let test_setup = TestSetup::new().await;
    let router = axum::Router::new()
        .nest("/feature-flag", generate_route::<crate::RestStateImpl>())
        .with_state(test_setup.rest_state.clone())
        // Empty user-context — production middleware would reject earlier
        // with 401, but the handler-internal path also returns 401 via
        // ServiceError::Unauthorized → error_handler mapping.
        .layer(Extension(None::<Arc<str>> as RestContext));

    let req = Request::builder()
        .method("GET")
        .uri("/feature-flag/totally_unknown_flag_xyz")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "unauth REST call must return 401"
    );
}
