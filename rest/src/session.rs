use std::sync::Arc;

use axum::extract::Request;
use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;
#[cfg(feature = "oidc")]
use axum_oidc::{EmptyAdditionalClaims, OidcClaims};
use service::session::SessionService;
use tower_cookies::Cookies;

pub type Context = Option<Arc<str>>;
use crate::RestStateDef;

/// Carries the real admin identity when a session is actively impersonating
/// another user.  Injected as an Axum request extension by both
/// `context_extractor` variants when `session.impersonate_user_id.is_some()`.
/// The effective [`Context`] (from [`resolve_session_user_id`]) is the TARGET
/// user — `RealUser` is the additional audit identity of the real admin.
///
/// Must implement `Clone + Send + Sync + 'static` for Axum extension storage.
#[derive(Clone, Debug)]
pub struct RealUser(pub Arc<str>);

/// Returns `Some(RealUser(session.user_id))` when the session is currently
/// impersonating another user (`impersonate_user_id` is `Some`).
/// Returns `None` for plain sessions (no impersonation active).
pub fn real_user_extension(session: &service::session::Session) -> Option<RealUser> {
    if session.impersonate_user_id.is_some() {
        Some(RealUser(session.user_id.clone()))
    } else {
        None
    }
}

/// Returns `true` when a request should emit an audit log line:
/// `real_user_present` is `true` (session is impersonating) AND `method` is
/// a mutating HTTP method (POST, PUT, PATCH, DELETE).
/// Returns `false` for GET, HEAD, OPTIONS, or when not impersonating.
pub fn should_audit_impersonated_write(method: &http::Method, real_user_present: bool) -> bool {
    real_user_present
        && matches!(
            *method,
            http::Method::POST
                | http::Method::PUT
                | http::Method::PATCH
                | http::Method::DELETE
        )
}

/// Tower middleware that emits a single structured `tracing::info!` line for
/// every mutating request (POST/PUT/PATCH/DELETE) made while an admin session
/// is impersonating another user (SC3: no write stays unattributed).
///
/// **Mounting order (important):** This middleware must be placed IMMEDIATELY
/// BEFORE the `context_extractor` layer in the `.layer(...)` chain in
/// `rest/src/lib.rs`.  Tower applies layers in reverse source order (last-added
/// layer = outermost, runs first).  Placing this layer before `context_extractor`
/// in source order means it is the inner layer, so `context_extractor` (outer)
/// runs first — populating `Context` + `RealUser` — and then this middleware
/// sees them when it executes.
pub async fn audit_impersonated_writes(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    if should_audit_impersonated_write(&method, request.extensions().get::<RealUser>().is_some()) {
        let real_user_id: Arc<str> = request
            .extensions()
            .get::<RealUser>()
            .map(|r| r.0.clone())
            .unwrap_or_else(|| Arc::from("<unknown>"));
        let acting_as: Arc<str> = request
            .extensions()
            .get::<Context>()
            .and_then(|c| c.clone())
            .unwrap_or_else(|| Arc::from("<none>"));
        tracing::info!(
            real_user = %real_user_id,
            acting_as = %acting_as,
            method = %method,
            path = path,
            "impersonated write"
        );
    }

    next.run(request).await
}

#[cfg(feature = "oidc")]
pub async fn register_session<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    claims: Option<OidcClaims<EmptyAdditionalClaims>>,
    request: Request,
    next: Next,
) -> Response {
    use time::OffsetDateTime;
    use tower_cookies::Cookie;

    let cookies = request
        .extensions()
        .get::<Cookies>()
        .expect("Cookies extension not set");

    if let Some(oidc_claims) = claims {
        let username = oidc_claims
            .preferred_username()
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|| "NoUsername".to_string());
        let session = rest_state
            .session_service()
            .new_session_for_user(&username)
            .await
            .unwrap();
        let session_id = session.id.to_string();
        let now = OffsetDateTime::now_utc();
        let expires = now + time::Duration::days(365);
        let cookie = Cookie::build(Cookie::new("app_session", session_id))
            .path("/")
            .expires(expires)
            .http_only(true)
            .same_site(tower_cookies::cookie::SameSite::Strict)
            .secure(true);
        cookies.add(cookie.into());
    }
    next.run(request).await
}

fn resolve_session_user_id(session: &service::session::Session) -> Option<Arc<str>> {
    if let Some(ref impersonate_user_id) = session.impersonate_user_id {
        Some(impersonate_user_id.clone())
    } else {
        Some(session.user_id.clone())
    }
}

#[cfg(feature = "oidc")]
pub async fn context_extractor<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    mut request: Request,
    next: Next,
) -> Response {
    let cookies = request
        .extensions()
        .get::<Cookies>()
        .expect("Cookies extension not set");
    tracing::info!("All cookies: {:?}", cookies.list());

    tracing::info!("Search for app_session cookie");
    if let Some(cookie) = cookies.get("app_session") {
        tracing::info!("app_session cookie found: {:?}", cookie);
        let session_id = cookie.value();
        tracing::info!("Session ID: {:?}", session_id);
        if let Some(session) = rest_state
            .session_service()
            .verify_user_session(session_id)
            .await
            .unwrap()
        {
            tracing::info!("Session found: {:?}", session);
            request
                .extensions_mut()
                .insert(resolve_session_user_id(&session));
            // D-32-01: inject RealUser when impersonating; effective Context is
            // unchanged (resolve_session_user_id returns the target user).
            if let Some(real_user) = real_user_extension(&session) {
                request.extensions_mut().insert(real_user);
            }
        } else {
            tracing::info!("Session not found");
            request.extensions_mut().insert(None::<Arc<str>>);
        }
    } else {
        tracing::info!("app_session cookie not found");
        request.extensions_mut().insert(None::<Arc<str>>);
    };
    next.run(request).await
}

#[cfg(all(feature = "mock_auth", not(feature = "oidc")))]
pub async fn context_extractor<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    mut request: Request,
    next: Next,
) -> Response {
    use time::OffsetDateTime;
    use tower_cookies::Cookie;

    let cookies = request
        .extensions()
        .get::<Cookies>()
        .expect("Cookies extension not set");

    if let Some(cookie) = cookies.get("app_session") {
        let session_id = cookie.value();
        if let Some(session) = rest_state
            .session_service()
            .verify_user_session(session_id)
            .await
            .unwrap()
        {
            request
                .extensions_mut()
                .insert(resolve_session_user_id(&session));
            // D-32-01: inject RealUser when impersonating; effective Context
            // unchanged (resolve_session_user_id returns the target user).
            if let Some(real_user) = real_user_extension(&session) {
                request.extensions_mut().insert(real_user);
            }
        } else {
            // Session expired or invalid — create a new one for DEVUSER
            let session = rest_state
                .session_service()
                .new_session_for_user("DEVUSER")
                .await
                .unwrap();
            let now = OffsetDateTime::now_utc();
            let expires = now + time::Duration::days(365);
            let cookie = Cookie::build(Cookie::new("app_session", session.id.to_string()))
                .path("/")
                .expires(expires)
                .http_only(true)
                .same_site(tower_cookies::cookie::SameSite::Strict)
                .secure(true);
            cookies.add(cookie.into());
            request
                .extensions_mut()
                .insert(resolve_session_user_id(&session));
            // D-32-01: fresh DEVUSER sessions have no impersonation; returns
            // None harmlessly.
            if let Some(real_user) = real_user_extension(&session) {
                request.extensions_mut().insert(real_user);
            }
        }
    } else {
        // No session cookie — auto-create session for DEVUSER
        let session = rest_state
            .session_service()
            .new_session_for_user("DEVUSER")
            .await
            .unwrap();
        let now = OffsetDateTime::now_utc();
        let expires = now + time::Duration::days(365);
        let cookie = Cookie::build(Cookie::new("app_session", session.id.to_string()))
            .path("/")
            .expires(expires)
            .http_only(true)
            .same_site(tower_cookies::cookie::SameSite::Strict)
            .secure(true);
        cookies.add(cookie.into());
        request
            .extensions_mut()
            .insert(resolve_session_user_id(&session));
        // D-32-01: fresh DEVUSER sessions have no impersonation; returns
        // None harmlessly.
        if let Some(real_user) = real_user_extension(&session) {
            request.extensions_mut().insert(real_user);
        }
    };
    next.run(request).await
}

#[cfg(all(feature = "mock_auth", not(feature = "oidc")))]
pub async fn forbid_unauthenticated<RestState: RestStateDef>(
    State(_rest_state): State<RestState>,
    request: Request,
    next: Next,
) -> Response {
    next.run(request).await
}
#[cfg(feature = "oidc")]
pub async fn forbid_unauthenticated<RestState: RestStateDef>(
    State(_rest_state): State<RestState>,
    request: Request,
    next: Next,
) -> Response {
    use tracing::{info, warn};

    info!("Checking authentication");
    if request.extensions().get::<Context>().is_some()
        && request.extensions().get::<Context>().unwrap().is_some()
        || request.uri().path().ends_with("/ical")
        || request.uri().path().ends_with("/authenticate")
    {
        info!("Authenticated: {:?}", request.extensions().get::<Context>());
        next.run(request).await
    } else {
        warn!("Not atuhenticated");
        Response::builder()
            .status(401)
            .body("Unauthorized".into())
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_session(user_id: &str, impersonate_user_id: Option<&str>) -> service::session::Session {
        service::session::Session {
            id: Arc::from(Uuid::new_v4().to_string()),
            user_id: Arc::from(user_id),
            expires: 9_999_999_999,
            created: 0,
            impersonate_user_id: impersonate_user_id.map(Arc::from),
        }
    }

    // --- real_user_extension ---

    #[test]
    fn real_user_extension_present_when_impersonating() {
        let session = make_session("ADMIN", Some("TARGET"));
        let result = real_user_extension(&session);
        assert!(result.is_some(), "expected RealUser extension when impersonating");
        assert_eq!(
            result.unwrap().0.as_ref(),
            "ADMIN",
            "RealUser must carry the real admin identity"
        );
    }

    #[test]
    fn real_user_extension_absent_when_not_impersonating() {
        let session = make_session("ADMIN", None);
        let result = real_user_extension(&session);
        assert!(result.is_none(), "expected no RealUser extension for a plain session");
    }

    // --- should_audit_impersonated_write ---

    #[test]
    fn audit_post_while_impersonating() {
        assert!(should_audit_impersonated_write(&http::Method::POST, true));
    }

    #[test]
    fn audit_put_while_impersonating() {
        assert!(should_audit_impersonated_write(&http::Method::PUT, true));
    }

    #[test]
    fn audit_patch_while_impersonating() {
        assert!(should_audit_impersonated_write(&http::Method::PATCH, true));
    }

    #[test]
    fn audit_delete_while_impersonating() {
        assert!(should_audit_impersonated_write(&http::Method::DELETE, true));
    }

    #[test]
    fn no_audit_for_get_even_while_impersonating() {
        assert!(!should_audit_impersonated_write(&http::Method::GET, true));
    }

    #[test]
    fn no_audit_for_head_even_while_impersonating() {
        assert!(!should_audit_impersonated_write(&http::Method::HEAD, true));
    }

    #[test]
    fn no_audit_for_options_even_while_impersonating() {
        assert!(!should_audit_impersonated_write(&http::Method::OPTIONS, true));
    }

    #[test]
    fn no_audit_for_post_when_not_impersonating() {
        assert!(!should_audit_impersonated_write(&http::Method::POST, false));
    }

    #[test]
    fn no_audit_for_delete_when_not_impersonating() {
        assert!(!should_audit_impersonated_write(&http::Method::DELETE, false));
    }
}
