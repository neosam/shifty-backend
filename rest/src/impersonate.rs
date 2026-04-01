use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{delete, get, post},
    Extension, Router,
};
use rest_types::ImpersonateTO;
use service::{
    permission::{Authentication, PermissionService},
    session::SessionService,
};
use tower_cookies::Cookies;
use utoipa::OpenApi;

use crate::{error_handler, RestStateDef};

#[derive(OpenApi)]
#[openapi(
    tags((name = "Impersonate", description = "Admin user impersonation endpoints")),
    paths(start_impersonate, stop_impersonate, get_impersonate_status),
)]
pub struct ImpersonateApiDoc;

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_impersonate_status::<RestState>))
        .route("/", delete(stop_impersonate::<RestState>))
        .route("/{user_id}", post(start_impersonate::<RestState>))
}

fn get_session_id_from_cookies(cookies: &Cookies) -> Option<String> {
    cookies.get("app_session").map(|c| c.value().to_string())
}

#[utoipa::path(
    post,
    path = "/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID to impersonate"),
    ),
    responses(
        (status = 200, description = "Impersonation started", body = ImpersonateTO),
        (status = 403, description = "Forbidden - not an admin"),
        (status = 404, description = "Target user not found"),
    ),
    tag = "Impersonate"
)]
pub async fn start_impersonate<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(cookies): Extension<Cookies>,
    Path(target_user_id): Path<String>,
) -> Response {
    error_handler(
        (async {
            let session_id = get_session_id_from_cookies(&cookies)
                .ok_or(service::ServiceError::Unauthorized)?;
            let session = rest_state
                .session_service()
                .verify_user_session(&session_id)
                .await?
                .ok_or(service::ServiceError::Unauthorized)?;

            // Check admin privilege against the REAL user, not the impersonated one
            let real_user_context: Authentication<Option<Arc<str>>> =
                Authentication::Context(Some(session.user_id.clone()));
            rest_state
                .permission_service()
                .check_permission("admin", real_user_context)
                .await?;

            // Verify target user exists
            let target_exists = rest_state
                .permission_service()
                .user_exists(&target_user_id, Authentication::Full)
                .await?;
            if !target_exists {
                return Err(service::ServiceError::EntityNotFoundGeneric(
                    Arc::from(target_user_id.as_str()),
                )
                .into());
            }

            rest_state
                .session_service()
                .start_impersonate(session.id.clone(), Arc::from(target_user_id.as_str()))
                .await?;

            let response = serde_json::to_string(&ImpersonateTO {
                impersonating: true,
                user_id: Some(Arc::from(target_user_id.as_str())),
            })
            .unwrap();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(response))
                .unwrap())
        })
        .await,
    )
}

#[utoipa::path(
    delete,
    path = "/",
    responses(
        (status = 200, description = "Impersonation stopped", body = ImpersonateTO),
        (status = 403, description = "Forbidden - not an admin"),
    ),
    tag = "Impersonate"
)]
pub async fn stop_impersonate<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(cookies): Extension<Cookies>,
) -> Response {
    error_handler(
        (async {
            let session_id = get_session_id_from_cookies(&cookies)
                .ok_or(service::ServiceError::Unauthorized)?;
            let session = rest_state
                .session_service()
                .verify_user_session(&session_id)
                .await?
                .ok_or(service::ServiceError::Unauthorized)?;

            // Check admin privilege against the REAL user
            let real_user_context: Authentication<Option<Arc<str>>> =
                Authentication::Context(Some(session.user_id.clone()));
            rest_state
                .permission_service()
                .check_permission("admin", real_user_context)
                .await?;

            rest_state
                .session_service()
                .stop_impersonate(session.id.clone())
                .await?;

            let response = serde_json::to_string(&ImpersonateTO {
                impersonating: false,
                user_id: None,
            })
            .unwrap();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(response))
                .unwrap())
        })
        .await,
    )
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Current impersonation status", body = ImpersonateTO),
        (status = 403, description = "Forbidden - not an admin"),
    ),
    tag = "Impersonate"
)]
pub async fn get_impersonate_status<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(cookies): Extension<Cookies>,
) -> Response {
    error_handler(
        (async {
            let session_id = get_session_id_from_cookies(&cookies)
                .ok_or(service::ServiceError::Unauthorized)?;
            let session = rest_state
                .session_service()
                .verify_user_session(&session_id)
                .await?
                .ok_or(service::ServiceError::Unauthorized)?;

            // Check admin privilege against the REAL user
            let real_user_context: Authentication<Option<Arc<str>>> =
                Authentication::Context(Some(session.user_id.clone()));
            rest_state
                .permission_service()
                .check_permission("admin", real_user_context)
                .await?;

            let response = serde_json::to_string(&ImpersonateTO {
                impersonating: session.impersonate_user_id.is_some(),
                user_id: session.impersonate_user_id.clone(),
            })
            .unwrap();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(response))
                .unwrap())
        })
        .await,
    )
}
