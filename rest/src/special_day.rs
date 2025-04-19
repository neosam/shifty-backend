use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use rest_types::SpecialDayTO;
use std::sync::Arc;
use tracing::instrument;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};
use service::special_days::SpecialDayService;

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route(
            "/for-week/{year}/{calendar_week}",
            get(get_special_days_for_week::<RestState>),
        )
        .route("/", post(create_special_days::<RestState>))
        .route("/{id}", delete(delete_special_day::<RestState>))
}

#[instrument(skip(rest_state))]
pub async fn get_special_days_for_week<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((year, week)): Path<(u32, u8)>,
) -> Response {
    error_handler(
        (async {
            let special_days: Arc<[SpecialDayTO]> = rest_state
                .special_day_service()
                .get_by_week(year, week, context.into())
                .await?
                .iter()
                .map(SpecialDayTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&special_days).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn create_special_days<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(special_day): Json<SpecialDayTO>,
) -> Response {
    error_handler(
        (async {
            let special_day = SpecialDayTO::from(
                &rest_state
                    .special_day_service()
                    .create(&(&special_day).into(), context.into())
                    .await?,
            );
            Ok(Response::builder()
                .status(201)
                .body(Body::new(serde_json::to_string(&special_day).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn delete_special_day<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(special_day_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .special_day_service()
                .delete(special_day_id, context.into())
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}
