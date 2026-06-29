//! Phase 32 Plan 32-01 — Backend integration tests for the admin-impersonation
//! audit layer.
//!
//! # Coverage
//!
//! * **SC5 / T-32-01:** A non-admin caller is rejected with 403 when attempting
//!   to start impersonation; the DEVUSER admin succeeds with 200.
//! * **RealUser inject / SC3 wiring:** After `start_impersonate`, the
//!   `context_extractor` middleware injects `Context = TARGET` (the impersonated
//!   user) and `Extension<RealUser> = DEVUSER` (the real admin) into request
//!   extensions.  Without impersonation the Context equals the real user and no
//!   RealUser extension is present.  A POST probe is used so the mutating-method
//!   path (SC3) is exercised end-to-end.
//! * **P10 / D-32-02a:** The admin can STOP impersonation while impersonating a
//!   non-admin (DELETE /admin/impersonate → 200) because the handler reads the
//!   raw `session.user_id` for the admin check.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use dao::PermissionDao;
use dao_impl_sqlite::PermissionDaoImpl;
use http_body_util::BodyExt;
use rest::impersonate::generate_route;
use rest::{Context as RestContext, RealUser, RestStateDef};
use service::session::SessionService;
use tower::ServiceExt;
use tower_cookies::CookieManagerLayer;

use crate::integration_test::TestSetup;

// ── helpers ────────────────────────────────────────────────────────────────────

/// Creates a user WITHOUT any role (non-admin).
async fn create_plain_user(test_setup: &TestSetup, name: &str) {
    let permission_dao = PermissionDaoImpl::new(test_setup.pool.clone());
    permission_dao
        .create_user(&dao::UserEntity { name: name.into() }, "test")
        .await
        .unwrap_or_else(|_| panic!("Expected to create user '{}'", name));
}

/// Sends a single request to the given router with a session cookie and
/// returns the response.  The router is cloned so tests can reuse it.
async fn send_req(
    router: &axum::Router,
    method: &str,
    path: &str,
    session_id: &str,
) -> axum::http::Response<Body> {
    let req = Request::builder()
        .method(method)
        .uri(path)
        .header("Cookie", format!("app_session={}", session_id))
        .body(Body::empty())
        .unwrap();
    router.clone().oneshot(req).await.unwrap()
}

/// Collects the response body into a UTF-8 `String`.
async fn body_text(response: axum::http::Response<Body>) -> String {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

/// Builds a minimal router that nests the real `impersonate::generate_route`
/// with only a `CookieManagerLayer`.  Used for Tests 1 and 3.
fn build_impersonate_router(test_setup: &TestSetup) -> axum::Router {
    axum::Router::new()
        .nest("/admin/impersonate", generate_route::<crate::RestStateImpl>())
        .with_state(test_setup.rest_state.clone())
        .layer(CookieManagerLayer::new())
}

/// Handler that encodes the current `Context` and optional `RealUser`
/// extension into the response body as `"context=<val> real_user=<val>"`.
/// Used by Test 2 to inspect what `context_extractor` injected.
async fn probe_handler(
    context: Option<axum::Extension<RestContext>>,
    real_user: Option<axum::Extension<RealUser>>,
) -> axum::http::Response<Body> {
    let context_str = context
        .and_then(|e| e.0.clone())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "<none>".to_string());
    let real_user_str = real_user
        .map(|e| e.0 .0.to_string())
        .unwrap_or_else(|| "<none>".to_string());
    axum::http::Response::builder()
        .status(200)
        .body(Body::from(format!(
            "context={} real_user={}",
            context_str, real_user_str
        )))
        .unwrap()
}

/// Builds a router with a `/probe` POST+GET endpoint wired through the real
/// `context_extractor` and `CookieManagerLayer`.  Used by Test 2.
fn build_probe_router(test_setup: &TestSetup) -> axum::Router {
    use axum::middleware;

    axum::Router::new()
        .route("/probe", axum::routing::post(probe_handler))
        .route("/probe", axum::routing::get(probe_handler))
        // context_extractor is the outer (first-to-run) layer because
        // Tower applies layers in reverse source order.
        .layer(middleware::from_fn_with_state(
            test_setup.rest_state.clone(),
            rest::session::context_extractor::<crate::RestStateImpl>,
        ))
        .layer(CookieManagerLayer::new())
}

// ── tests ──────────────────────────────────────────────────────────────────────

/// SC5 / T-32-01: admin gate rejects non-admin caller with 403; DEVUSER
/// admin succeeds with 200.
#[tokio::test]
async fn sc5_non_admin_cannot_start_impersonation() {
    let test_setup = TestSetup::new().await;

    // NOBODY = non-admin user; TARGET = user to impersonate
    create_plain_user(&test_setup, "NOBODY").await;
    create_plain_user(&test_setup, "TARGET").await;

    // Mint separate sessions
    let nobody_session = test_setup
        .rest_state
        .session_service()
        .new_session_for_user("NOBODY")
        .await
        .unwrap();
    let devuser_session = test_setup
        .rest_state
        .session_service()
        .new_session_for_user("DEVUSER")
        .await
        .unwrap();

    let router = build_impersonate_router(&test_setup);

    // Non-admin → 403
    let resp = send_req(&router, "POST", "/admin/impersonate/TARGET", &nobody_session.id).await;
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "non-admin must receive 403 (SC5)"
    );

    // Admin → 200
    let resp = send_req(
        &router,
        "POST",
        "/admin/impersonate/TARGET",
        &devuser_session.id,
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "admin must receive 200 when starting impersonation"
    );
}

/// RealUser inject + effective context (SC3 wiring):
///
/// Under an active impersonation the probe sees `Context = TARGET` and
/// `RealUser = DEVUSER`.  Without impersonation the probe sees
/// `Context = DEVUSER` and no RealUser extension.
///
/// A POST probe asserts the *inputs* the audit middleware consumes are present
/// for a mutating request under impersonation: `Context = TARGET` + the
/// `RealUser = DEVUSER` extension. (WR-03) The probe router does NOT include
/// `audit_impersonated_writes` itself, so this does not assert the `tracing`
/// emission end-to-end — that emission decision is covered by the
/// `should_audit_impersonated_write` method-truth-table unit tests in
/// `rest::session`. Together they cover SC3: inputs present here, emit-logic there.
#[tokio::test]
async fn real_user_injected_under_impersonation() {
    let test_setup = TestSetup::new().await;

    create_plain_user(&test_setup, "TARGET").await;

    // Admin session that will be used to impersonate TARGET
    let admin_session = test_setup
        .rest_state
        .session_service()
        .new_session_for_user("DEVUSER")
        .await
        .unwrap();

    // Activate impersonation at the service level (bypasses REST; mirrors what
    // POST /admin/impersonate/TARGET does internally).
    test_setup
        .rest_state
        .session_service()
        .start_impersonate(admin_session.id.clone(), Arc::from("TARGET"))
        .await
        .unwrap();

    let router = build_probe_router(&test_setup);

    // POST probe with impersonating session: Context must be TARGET, RealUser must be DEVUSER
    let resp = send_req(&router, "POST", "/probe", &admin_session.id).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_text(resp).await;
    assert!(
        body.contains("context=TARGET"),
        "effective context must be TARGET under impersonation, got: {body}"
    );
    assert!(
        body.contains("real_user=DEVUSER"),
        "RealUser must carry DEVUSER (real admin) under impersonation, got: {body}"
    );

    // Control: non-impersonating session
    let plain_session = test_setup
        .rest_state
        .session_service()
        .new_session_for_user("DEVUSER")
        .await
        .unwrap();
    let resp = send_req(&router, "POST", "/probe", &plain_session.id).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_text(resp).await;
    assert!(
        body.contains("context=DEVUSER"),
        "plain session must set Context = DEVUSER, got: {body}"
    );
    assert!(
        body.contains("real_user=<none>"),
        "plain session must have no RealUser, got: {body}"
    );
}

/// P10 / D-32-02a: while impersonating a non-admin the admin can still stop
/// impersonation because `stop_impersonate` reads the raw `session.user_id`
/// for the admin check — not the impersonated (non-admin) Context.
#[tokio::test]
async fn p10_stop_works_while_impersonating_non_admin() {
    let test_setup = TestSetup::new().await;

    create_plain_user(&test_setup, "NOBODY").await;

    let admin_session = test_setup
        .rest_state
        .session_service()
        .new_session_for_user("DEVUSER")
        .await
        .unwrap();

    let router = build_impersonate_router(&test_setup);

    // Start impersonating NOBODY (non-admin)
    let resp = send_req(
        &router,
        "POST",
        "/admin/impersonate/NOBODY",
        &admin_session.id,
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "admin must be able to start impersonating a non-admin"
    );

    // Stop must succeed via raw session.user_id (P10)
    let resp = send_req(&router, "DELETE", "/admin/impersonate", &admin_session.id).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "stop must succeed while impersonating a non-admin (P10 / D-32-02a)"
    );
}
