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
    use http::header::SET_COOKIE;
    use service::session::SessionService;

    let mut response = next.run(request).await;

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
        let cookie = format!("session={} Path=/; HttpOnly; Secure", session.id);
        response
            .headers_mut()
            .append(SET_COOKIE, cookie.parse().unwrap());
    }
    response
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

    if let Some(cookie) = cookies.get("app_session") {
        if let Some(session) = rest_state
            .session_service()
            .verify_user_session(cookie.value())
            .await
            .unwrap()
        {
            request.extensions_mut().insert(Some(session.user_id));
        } else {
            request.extensions_mut().insert(None::<Arc<str>>);
        }
    } else {
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
