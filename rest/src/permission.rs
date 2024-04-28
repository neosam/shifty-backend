use axum::{body::Body, extract::State, response::Response, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error_handler, RestStateDef};
use service::PermissionService;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(default)]
    pub id: Uuid,
    pub name: String,
}

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/user/", post(add_user::<RestState>))
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
                .status(200)
                .body(Body::from(""))
                .unwrap())
        })
        .await,
    )
}
