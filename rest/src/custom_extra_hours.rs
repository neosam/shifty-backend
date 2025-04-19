use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    Extension, Json,
};
use rest_types::CustomExtraHoursTO;
use service::custom_extra_hours::CustomExtraHoursService;
use tracing::instrument;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> axum::Router<RestState> {
    axum::Router::new()
        .route("/", axum::routing::get(get_all::<RestState>))
        .route("/{id}", axum::routing::get(get_by_id::<RestState>))
        .route("/", axum::routing::post(create::<RestState>))
        .route("/{id}", axum::routing::put(update::<RestState>))
        .route("/{id}", axum::routing::delete(delete::<RestState>))
}

#[instrument(skip(rest_state))]
pub async fn get_all<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let weekly_summary: Arc<[CustomExtraHoursTO]> = rest_state
                .custom_extra_hours_service()
                .get_all(context.into(), None)
                .await?
                .iter()
                .map(CustomExtraHoursTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&weekly_summary).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn get_by_id<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    id: Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let custom_extra_hours: CustomExtraHoursTO = rest_state
                .custom_extra_hours_service()
                .get_by_id(*id, context.into(), None)
                .await?
                .into();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(
                    serde_json::to_string(&custom_extra_hours).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn create<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(custom_extra_hours): Json<CustomExtraHoursTO>,
) -> Response {
    error_handler(
        (async {
            let custom_extra_hours: CustomExtraHoursTO = rest_state
                .custom_extra_hours_service()
                .create(&custom_extra_hours.into(), context.into(), None)
                .await?
                .into();
            Ok(Response::builder()
                .status(201)
                .body(Body::new(
                    serde_json::to_string(&custom_extra_hours).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn update<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    id: Path<Uuid>,
    Json(custom_extra_hours): Json<CustomExtraHoursTO>,
) -> Response {
    error_handler(
        (async {
            let custom_extra_hours: CustomExtraHoursTO = rest_state
                .custom_extra_hours_service()
                .update(&custom_extra_hours.into(), context.into(), None)
                .await?
                .into();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(
                    serde_json::to_string(&custom_extra_hours).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn delete<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    id: Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .custom_extra_hours_service()
                .delete(*id, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}
