use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query};
use axum::routing::{delete, get, post, put};
use axum::{extract::State, response::Response};
use axum::{Extension, Json, Router};
use rest_types::{SalesPersonTO, SalesPersonUnavailableTO};
use serde::Deserialize;
use service::sales_person::SalesPersonService;
use service::sales_person_unavailable::SalesPersonUnavailableService;
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
        .route(
            "/:id/unavailable",
            get(get_sales_person_unavailable::<RestState>),
        )
        .route(
            "/unavailable",
            post(create_sales_person_unavailable::<RestState>),
        )
        .route(
            "/unavailable/:id",
            delete(delete_sales_person_unavailable::<RestState>),
        )
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

#[derive(Clone, Debug, Deserialize)]
pub struct ReportRequest {
    year: Option<u32>,
    calendar_week: Option<u8>,
}

pub async fn get_sales_person_unavailable<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
    query: Query<ReportRequest>,
) -> Response {
    error_handler(
        (async {
            let tos = if let (Some(year), Some(calendar_week)) = (query.year, query.calendar_week) {
                rest_state
                    .sales_person_unavailable_service()
                    .get_by_week_for_sales_person(
                        sales_person_id,
                        year,
                        calendar_week,
                        context.into(),
                    )
                    .await?
                    .iter()
                    .map(SalesPersonUnavailableTO::from)
                    .collect::<Vec<_>>()
            } else {
                rest_state
                    .sales_person_unavailable_service()
                    .get_all_for_sales_person(sales_person_id, context.into())
                    .await?
                    .iter()
                    .map(SalesPersonUnavailableTO::from)
                    .collect::<Vec<_>>()
            };
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&tos).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn create_sales_person_unavailable<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(sales_person_unavailable): Json<SalesPersonUnavailableTO>,
) -> Response {
    error_handler(
        (async {
            let unavailable = SalesPersonUnavailableTO::from(
                &rest_state
                    .sales_person_unavailable_service()
                    .create(&(&sales_person_unavailable).into(), context.into())
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&unavailable).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn delete_sales_person_unavailable<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(unavailable_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .sales_person_unavailable_service()
                .delete(unavailable_id, context.into())
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}
