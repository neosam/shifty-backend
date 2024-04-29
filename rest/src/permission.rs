use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    response::Response,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{error_handler, RestStateDef};
use service::PermissionService;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub name: String,
}
impl From<&service::User> for User {
    fn from(user: &service::User) -> Self {
        Self {
            name: user.name.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
}
impl From<&service::Role> for Role {
    fn from(role: &service::Role) -> Self {
        Self {
            name: role.name.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Privilege {
    pub name: String,
}
impl From<&service::Privilege> for Privilege {
    fn from(privilege: &service::Privilege) -> Self {
        Self {
            name: privilege.name.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRole {
    pub user: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RolePrivilege {
    pub role: String,
    pub privilege: String,
}

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/user/", get(get_all_users::<RestState>))
        .route("/user/", post(add_user::<RestState>))
        .route("/user/", delete(remove_user::<RestState>))
        .route("/role/", get(get_all_roles::<RestState>))
        .route("/role/", post(add_role::<RestState>))
        .route("/role/", delete(delete_role::<RestState>))
        .route("/privilege/", get(get_all_privileges::<RestState>))
        .route("/user-role/", post(add_user_role::<RestState>))
        .route("/user-role/", delete(remove_user_role::<RestState>))
        .route("/role-privilege/", post(add_role_privilege::<RestState>))
        .route(
            "/role-privilege/",
            delete(remove_role_privilege::<RestState>),
        )
}

pub async fn add_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Json(user): Json<User>,
) -> Response {
    println!("Adding user: {:?}", user);
    error_handler(
        (async {
            rest_state
                .permission_service()
                .create_user(user.name.as_str())
                .await?;
            Ok(Response::builder()
                .status(201)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

pub async fn remove_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Json(user): Json<String>,
) -> Response {
    println!("Removing user: {:?}", user);
    error_handler(
        (async {
            rest_state.permission_service().delete_user(&user).await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

pub async fn add_role<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Json(role): Json<Role>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .permission_service()
                .create_role(role.name.as_str())
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

pub async fn delete_role<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Json(role): Json<String>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .permission_service()
                .delete_role(role.as_str())
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

pub async fn add_user_role<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Json(user_role): Json<UserRole>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .permission_service()
                .add_user_role(user_role.user.as_str(), user_role.role.as_str())
                .await?;
            Ok(Response::builder()
                .status(201)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

pub async fn remove_user_role<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Json(user_role): Json<UserRole>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .permission_service()
                .delete_user_role(user_role.user.as_str(), user_role.role.as_str())
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}

pub async fn add_role_privilege<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Json(role_privilege): Json<RolePrivilege>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .permission_service()
                .add_role_privilege(
                    role_privilege.role.as_str(),
                    role_privilege.privilege.as_str(),
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

pub async fn remove_role_privilege<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Json(role_privilege): Json<RolePrivilege>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .permission_service()
                .delete_role_privilege(
                    role_privilege.role.as_str(),
                    role_privilege.privilege.as_str(),
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

pub async fn get_all_users<RestState: RestStateDef>(rest_state: State<RestState>) -> Response {
    error_handler(
        (async {
            let users: Arc<[User]> = rest_state
                .permission_service()
                .get_all_users()
                .await?
                .iter()
                .map(|u| User::from(u))
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::from(serde_json::to_string(&users).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn get_all_roles<RestState: RestStateDef>(rest_state: State<RestState>) -> Response {
    error_handler(
        (async {
            let roles: Arc<[Role]> = rest_state
                .permission_service()
                .get_all_roles()
                .await?
                .iter()
                .map(|u| Role::from(u))
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::from(serde_json::to_string(&roles).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn get_all_privileges<RestState: RestStateDef>(rest_state: State<RestState>) -> Response {
    error_handler(
        (async {
            let privileges: Arc<[Privilege]> = rest_state
                .permission_service()
                .get_all_privileges()
                .await?
                .iter()
                .map(|u| Privilege::from(u))
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::from(serde_json::to_string(&privileges).unwrap()))
                .unwrap())
        })
        .await,
    )
}
