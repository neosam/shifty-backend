use std::sync::Arc;

use axum::extract::Request;
use axum::extract::State;
use axum::middleware::Next;
use axum::response::Response;
#[cfg(feature = "oidc")]
use axum_oidc::{EmptyAdditionalClaims, OidcClaims};
#[cfg(feature = "mock_auth")]
use service::permission::MockContext;
use tower_cookies::Cookies;

#[cfg(feature = "mock_auth")]
pub type Context = MockContext;
#[cfg(feature = "oidc")]
pub type Context = Option<Arc<str>>;
use crate::RestStateDef;

#[cfg(feature = "oidc")]
pub async fn register_session<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    claims: Option<OidcClaims<EmptyAdditionalClaims>>,
    request: Request,
    next: Next,
) -> Response {
    use service::session::SessionService;
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
#[cfg(feature = "oidc")]
pub async fn context_extractor<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    mut request: Request,
    next: Next,
) -> Response {
    use service::session::SessionService;

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
            request.extensions_mut().insert(Some(session.user_id));
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
#[cfg(feature = "mock_auth")]
pub async fn context_extractor<RestState: RestStateDef>(
    mut request: Request,
    next: Next,
) -> Response {
    request.extensions_mut().insert(MockContext);
    next.run(request).await
}
