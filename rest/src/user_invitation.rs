use axum::extract::{Path, Request, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use service::permission::Authentication;
use service::user_invitation::{
    InvitationStatus as ServiceInvitationStatus, UserInvitationService,
};
use time::OffsetDateTime;
use tracing::instrument;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[cfg(feature = "oidc")]
use service::session::SessionService;
#[cfg(feature = "oidc")]
use tower_cookies::{Cookie, Cookies};

use crate::{error_handler, Context, RestStateDef};

// Re-export InvitationStatus with ToSchema for OpenAPI documentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum InvitationStatus {
    /// Invitation is valid and can be used
    Valid,
    /// Invitation has expired and cannot be used
    Expired,
    /// Invitation has already been redeemed
    Redeemed,
    /// Invitation session has been revoked
    #[serde(rename = "sessionrevoked")]
    SessionRevoked,
}

impl From<ServiceInvitationStatus> for InvitationStatus {
    fn from(status: ServiceInvitationStatus) -> Self {
        match status {
            ServiceInvitationStatus::Valid => InvitationStatus::Valid,
            ServiceInvitationStatus::Expired => InvitationStatus::Expired,
            ServiceInvitationStatus::Redeemed => InvitationStatus::Redeemed,
            ServiceInvitationStatus::SessionRevoked => InvitationStatus::SessionRevoked,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateInvitationRequest {
    /// Username of the user to invite
    pub username: String,
    /// Expiration time in hours (default: 168 hours = 7 days)
    pub expiration_hours: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InvitationResponse {
    /// Unique ID of the invitation
    pub id: Uuid,
    /// Username of the invited user
    pub username: String,
    /// Invitation token (UUID)
    pub token: Uuid,
    /// Complete invitation link URL
    pub invitation_link: String,
    /// When the invitation was redeemed (null if not yet redeemed)
    #[serde(with = "time::serde::rfc3339::option")]
    pub redeemed_at: Option<OffsetDateTime>,
    /// Current status of the invitation
    pub status: InvitationStatus,
}

#[cfg(feature = "oidc")]
pub async fn authenticate_with_invitation<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    Path(token): Path<Uuid>,
    request: Request,
) -> Response {
    let cookies = request
        .extensions()
        .get::<Cookies>()
        .expect("Cookies extension not set");

    match rest_state
        .user_invitation_service()
        .validate_and_consume_token(&token, None)
        .await
    {
        Ok(username) => {
            // OIDC mode: Create session and set authentication cookie
            match rest_state
                .session_service()
                .new_session_for_user(&username)
                .await
            {
                Ok(session) => {
                    let session_id = session.id.to_string();

                    // Mark the token as redeemed with the session ID
                    if let Err(_) = rest_state
                        .user_invitation_service()
                        .mark_token_redeemed(&token, &session_id, None)
                        .await
                    {
                        // Log error but don't fail the authentication
                        tracing::warn!("Failed to mark invitation token as redeemed");
                    }

                    let now = OffsetDateTime::now_utc();
                    let expires = now + time::Duration::days(365);
                    let cookie = Cookie::build(("app_session", session_id))
                        .path("/")
                        .expires(expires)
                        .http_only(true)
                        .same_site(tower_cookies::cookie::SameSite::Strict)
                        .secure(true);
                    cookies.add(cookie.into());
                    Redirect::to("/").into_response()
                }
                Err(_) => Response::builder()
                    .status(500)
                    .header("Content-Type", "text/plain")
                    .body("Failed to create session".into())
                    .unwrap(),
            }
        }
        Err(_) => Response::builder()
            .status(400)
            .header("Content-Type", "text/plain")
            .body("Invalid or expired invitation token".into())
            .unwrap(),
    }
}

#[cfg(all(feature = "mock_auth", not(feature = "oidc")))]
pub async fn authenticate_with_invitation<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    Path(token): Path<Uuid>,
    _request: Request,
) -> Response {
    match rest_state
        .user_invitation_service()
        .validate_and_consume_token(&token, None)
        .await
    {
        Ok(_username) => {
            // Mock auth mode: Mark token as redeemed with a mock session ID
            let mock_session_id = format!("mock-session-{}", uuid::Uuid::new_v4());
            if rest_state
                .user_invitation_service()
                .mark_token_redeemed(&token, &mock_session_id, None)
                .await.is_err()
            {
                tracing::warn!("Failed to mark invitation token as redeemed");
            }
            // Just redirect (authentication is bypassed globally)
            Redirect::to("/").into_response()
        }
        Err(_) => Response::builder()
            .status(400)
            .header("Content-Type", "text/plain")
            .body("Invalid or expired invitation token".into())
            .unwrap(),
    }
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    tags = ["User Invitations"],
    path = "/invitation",
    request_body = GenerateInvitationRequest,
    responses(
        (status = 200, description = "Invitation generated successfully", body = InvitationResponse),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden - admin privileges required"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn generate_invitation<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    Extension(auth_context): Extension<Context>,
    Json(request): Json<GenerateInvitationRequest>,
) -> Response {
    error_handler(
        (async {
            let expiration_hours = request.expiration_hours.unwrap_or(7 * 24); // Default to 7 days

            let invitation = rest_state
                .user_invitation_service()
                .generate_invitation(
                    &request.username,
                    expiration_hours,
                    None,
                    Authentication::Context(auth_context),
                )
                .await?;

            // Get the base URL from environment or config
            let base_url =
                std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
            let invitation_link = format!("{}/auth/invitation/{}", base_url, invitation.token);

            let response = InvitationResponse {
                id: invitation.id,
                username: invitation.username.to_string(),
                token: invitation.token,
                invitation_link,
                redeemed_at: invitation.redeemed_at,
                status: invitation.status.into(),
            };

            Ok(Json(response).into_response())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    tags = ["User Invitations"],
    path = "/invitation/user/{username}",
    params(
        ("username" = String, Path, description = "Username to list invitations for")
    ),
    responses(
        (status = 200, description = "List of user invitations", body = Vec<InvitationResponse>),
        (status = 403, description = "Forbidden - admin privileges required"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn list_user_invitations<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    Extension(auth_context): Extension<Context>,
    Path(username): Path<String>,
) -> Response {
    error_handler(
        (async {
            let invitations = rest_state
                .user_invitation_service()
                .list_invitations_for_user(&username, None, Authentication::Context(auth_context))
                .await?;

            let response: Vec<InvitationResponse> = invitations
                .into_iter()
                .map(|inv| {
                    let base_url = std::env::var("APP_URL")
                        .unwrap_or_else(|_| "http://localhost:3000".to_string());
                    let invitation_link = format!("{}/auth/invitation/{}", base_url, inv.token);
                    InvitationResponse {
                        id: inv.id,
                        username: inv.username,
                        token: inv.token,
                        invitation_link,
                        redeemed_at: inv.redeemed_at,
                        status: inv.status.into(),
                    }
                })
                .collect();
            Ok(Json(response).into_response())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    tags = ["User Invitations"],
    path = "/invitation/{id}",
    params(
        ("id" = Uuid, Path, description = "Invitation ID to revoke")
    ),
    responses(
        (status = 204, description = "Invitation revoked successfully"),
        (status = 403, description = "Forbidden - admin privileges required"),
        (status = 404, description = "Invitation not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn revoke_invitation<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    Extension(auth_context): Extension<Context>,
    Path(id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .user_invitation_service()
                .revoke_invitation(&id, None, Authentication::Context(auth_context))
                .await?;

            Ok(Response::builder()
                .status(204)
                .body(axum::body::Body::empty())
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    tags = ["User Invitations"],
    path = "/invitation/{id}/revoke-session",
    params(
        ("id" = Uuid, Path, description = "Invitation ID whose session should be revoked")
    ),
    responses(
        (status = 204, description = "Session revoked successfully"),
        (status = 403, description = "Forbidden - admin privileges required"),
        (status = 404, description = "Invitation not found or no session associated"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn revoke_session_for_invitation<RestState: RestStateDef>(
    State(rest_state): State<RestState>,
    Extension(auth_context): Extension<Context>,
    Path(id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .user_invitation_service()
                .revoke_session_for_invitation(&id, None, Authentication::Context(auth_context))
                .await?;

            Ok(Response::builder()
                .status(204)
                .body(axum::body::Body::empty())
                .unwrap())
        })
        .await,
    )
}

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/invitation", post(generate_invitation::<RestState>))
        .route(
            "/invitation/user/{username}",
            get(list_user_invitations::<RestState>),
        )
        .route(
            "/invitation/{id}",
            axum::routing::delete(revoke_invitation::<RestState>),
        )
        .route(
            "/invitation/{id}/revoke-session",
            post(revoke_session_for_invitation::<RestState>),
        )
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "User Invitations", description = "User invitation management API"),
    ),
    paths(
        generate_invitation,
        list_user_invitations,
        revoke_invitation,
        revoke_session_for_invitation,
    ),
    components(
        schemas(
            GenerateInvitationRequest,
            InvitationResponse,
            InvitationStatus,
        ),
    ),
)]
pub struct UserInvitationApiDoc;
