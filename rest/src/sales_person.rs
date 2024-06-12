use std::sync::Arc;

use axum::body::Body;
use axum::extract::Path;
use axum::routing::{delete, get, post, put};
use axum::{extract::State, response::Response};
use axum::{Extension, Json, Router};
use rest_types::SalesPersonTO;
use service::sales_person::SalesPersonService;
use uuid::Uuid;

use crate::{error_handler, Context, RestError, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_all_sales_persons::<RestState>))
        .route("/:id", get(get_sales_person::<RestState>))
        .route("/", post(create_sales_person::<RestState>))
        .route("/:id", put(update_sales_person::<RestState>))
        .route("/:id", delete(delete_sales_person::<RestState>))
        .route("/:id/user", get(get_sales_person_user::<RestState>))
        .route("/:id/user", post(set_sales_person_user::<RestState>))
        .route("/:id/user", delete(delete_sales_person_user::<RestState>))
        .route("/current", get(get_sales_person_current_user::<RestState>))
}

pub async fn get_all_sales_persons<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let sales_persons: Arc<[SalesPersonTO]> = rest_state
                .sales_person_service()
                .get_all(context.into())
                .await?
                .iter()
                .map(SalesPersonTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&sales_persons).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn get_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let sales_person = SalesPersonTO::from(
                &rest_state
                    .sales_person_service()
                    .get(sales_person_id, context.into())
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&sales_person).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn create_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(sales_person): Json<SalesPersonTO>,
) -> Response {
    error_handler(
        (async {
            let sales_person = SalesPersonTO::from(
                &rest_state
                    .sales_person_service()
                    .create(&(&sales_person).into(), context.into())
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&sales_person).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn update_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
    Json(sales_person): Json<SalesPersonTO>,
) -> Response {
    error_handler(
        (async {
            if sales_person_id != sales_person.id {
                return Err(RestError::InconsistentId(sales_person_id, sales_person.id));
            }
            rest_state
                .sales_person_service()
                .update(&(&sales_person).into(), context.into())
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&sales_person).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn delete_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .sales_person_service()
                .delete(sales_person_id, context.into())
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

pub async fn get_sales_person_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let user = rest_state
                .sales_person_service()
                .get_assigned_user(sales_person_id, context.into())
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&user).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn set_sales_person_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
    Json(user): Json<Arc<str>>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .sales_person_service()
                .set_user(sales_person_id, user.into(), context.into())
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

pub async fn delete_sales_person_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .sales_person_service()
                .set_user(sales_person_id, None, context.into())
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

pub async fn get_sales_person_current_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let sales_person = rest_state
                .sales_person_service()
                .get_sales_person_current_user(context.into())
                .await?
                .map(|sales_person| SalesPersonTO::from(&sales_person));

            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&sales_person).unwrap()))
                .unwrap())
        })
        .await,
    )
}
