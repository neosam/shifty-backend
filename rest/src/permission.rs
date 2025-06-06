use std::sync::Arc;

use rest_types::*;

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use tracing::instrument;
use utoipa::OpenApi;

use crate::{error_handler, Context, RestStateDef};
use service::PermissionService;

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/user", get(get_all_users::<RestState>))
        .route("/user", post(add_user::<RestState>))
        .route("/user/", delete(remove_user::<RestState>))
        .route("/role", get(get_all_roles::<RestState>))
        .route("/role", post(add_role::<RestState>))
        .route("/role", delete(delete_role::<RestState>))
        .route("/user/{user}/roles", get(get_roles_for_user::<RestState>))
        .route("/privilege/", get(get_all_privileges::<RestState>))
        .route("/user-role", post(add_user_role::<RestState>))
        .route("/user-role", delete(remove_user_role::<RestState>))
        .route("/role-privilege/", post(add_role_privilege::<RestState>))
        .route(
            "/role-privilege/",
            delete(remove_role_privilege::<RestState>),
        )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/user",
    tags = ["Permission"],
    request_body = UserTO,
    responses(
        (status = 201, description = "User created successfully"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn add_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(user): Json<UserTO>,
) -> Response {
    println!("Adding user: {:?}", user);
    error_handler(
        (async {
            rest_state
                .permission_service()
                .create_user(user.name.as_str(), context.into())
                .await?;
            Ok(Response::builder()
                .status(201)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/user/",
    tags = ["Permission"],
    request_body = String,
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn remove_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(user): Json<String>,
) -> Response {
    println!("Removing user: {:?}", user);
    error_handler(
        (async {
            rest_state
                .permission_service()
                .delete_user(&user, context.into())
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/role",
    tags = ["Permission"],
    request_body = RoleTO,
    responses(
        (status = 200, description = "Role created successfully"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn add_role<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(role): Json<RoleTO>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .permission_service()
                .create_role(role.name.as_str(), context.into())
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/role",
    tags = ["Permission"],
    request_body = String,
    responses(
        (status = 200, description = "Role deleted successfully"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn delete_role<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(role): Json<String>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .permission_service()
                .delete_role(role.as_str(), context.into())
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/user-role",
    tags = ["Permission"],
    request_body = UserRole,
    responses(
        (status = 201, description = "User role assigned successfully"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn add_user_role<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(user_role): Json<UserRole>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .permission_service()
                .add_user_role(
                    user_role.user.as_str(),
                    user_role.role.as_str(),
                    context.into(),
                )
                .await?;
            Ok(Response::builder()
                .status(201)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/user-role",
    tags = ["Permission"],
    request_body = UserRole,
    responses(
        (status = 200, description = "User role removed successfully"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn remove_user_role<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(user_role): Json<UserRole>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .permission_service()
                .delete_user_role(
                    user_role.user.as_str(),
                    user_role.role.as_str(),
                    context.into(),
                )
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/role-privilege/",
    tags = ["Permission"],
    request_body = RolePrivilege,
    responses(
        (status = 201, description = "Role privilege assigned successfully"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn add_role_privilege<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(role_privilege): Json<RolePrivilege>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .permission_service()
                .add_role_privilege(
                    role_privilege.role.as_str(),
                    role_privilege.privilege.as_str(),
                    context.into(),
                )
                .await?;
            Ok(Response::builder()
                .status(201)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/role-privilege/",
    tags = ["Permission"],
    request_body = RolePrivilege,
    responses(
        (status = 200, description = "Role privilege removed successfully"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn remove_role_privilege<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(role_privilege): Json<RolePrivilege>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .permission_service()
                .delete_role_privilege(
                    role_privilege.role.as_str(),
                    role_privilege.privilege.as_str(),
                    context.into(),
                )
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/user",
    tags = ["Permission"],
    responses(
        (status = 200, description = "List all users", body = [UserTO]),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_all_users<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let users: Arc<[UserTO]> = rest_state
                .permission_service()
                .get_all_users(context.into())
                .await?
                .iter()
                .map(UserTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::from(serde_json::to_string(&users).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/role",
    tags = ["Permission"],
    responses(
        (status = 200, description = "List all roles", body = [RoleTO]),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_all_roles<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let roles: Arc<[RoleTO]> = rest_state
                .permission_service()
                .get_all_roles(context.into())
                .await?
                .iter()
                .map(RoleTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::from(serde_json::to_string(&roles).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/privilege/",
    tags = ["Permission"],
    responses(
        (status = 200, description = "List all privileges", body = [PrivilegeTO]),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_all_privileges<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let privileges: Arc<[PrivilegeTO]> = rest_state
                .permission_service()
                .get_all_privileges(context.into())
                .await?
                .iter()
                .map(PrivilegeTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::from(serde_json::to_string(&privileges).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/user/{user}/roles",
    tags = ["Permission"],
    params(
        ("user", description = "User name", example = "john_doe")
    ),
    responses(
        (status = 200, description = "List roles for user", body = [RoleTO]),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_roles_for_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(user_id): Path<String>,
) -> Response {
    error_handler(
        (async {
            let roles: Arc<[RoleTO]> = rest_state
                .permission_service()
                .get_roles_for_user(&user_id, context.into())
                .await?
                .iter()
                .map(RoleTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::from(serde_json::to_string(&roles).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Permission", description = "Permission Management API"),
    ),
    paths(
        add_user,
        remove_user,
        add_role,
        delete_role,
        add_user_role,
        remove_user_role,
        add_role_privilege,
        remove_role_privilege,
        get_all_users,
        get_all_roles,
        get_all_privileges,
        get_roles_for_user,
    ),
    components(
        schemas(
            UserTO,
            RoleTO,
            PrivilegeTO,
            UserRole,
            RolePrivilege,
        ),
    ),
)]
pub struct PermissionApiDoc;
