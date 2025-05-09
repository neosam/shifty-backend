use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{get, post, put},
    Extension, Json, Router,
};
use rest_types::SlotTO;
use service::slot::SlotService;
use tracing::instrument;
use uuid::Uuid;

use crate::{error_handler, Context, RestError, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_all_slots::<RestState>))
        .route("/{id}", get(get_slot::<RestState>))
        .route("/week/{year}/{month}", get(get_slots_for_week::<RestState>))
        .route("/", post(create_slot::<RestState>))
        .route("/{id}", put(update_slot::<RestState>))
}

#[instrument(skip(rest_state))]
pub async fn get_all_slots<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let slots: Arc<[SlotTO]> = rest_state
                .slot_service()
                .get_slots(context.into(), None)
                .await?
                .iter()
                .map(SlotTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&slots).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn get_slot<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(slot_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let slot = SlotTO::from(
                &rest_state
                    .slot_service()
                    .get_slot(&slot_id, context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&slot).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn get_slots_for_week<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((year, week)): Path<(u32, u8)>,
) -> Response {
    error_handler(
        (async {
            let slots: Arc<[SlotTO]> = rest_state
                .slot_service()
                .get_slots_for_week(year, week, context.into(), None)
                .await?
                .iter()
                .map(SlotTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&slots).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn create_slot<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(slot): Json<SlotTO>,
) -> Response {
    error_handler(
        (async {
            let slot = SlotTO::from(
                &rest_state
                    .slot_service()
                    .create_slot(&(&slot).into(), context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&slot).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn update_slot<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(slot_id): Path<Uuid>,
    Json(slot): Json<SlotTO>,
) -> Response {
    error_handler(
        (async {
            if slot_id != slot.id {
                return Err(RestError::InconsistentId(slot_id, slot.id));
            }
            rest_state
                .slot_service()
                .update_slot(&(&slot).into(), context.into(), None)
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&slot).unwrap()))
                .unwrap())
        })
        .await,
    )
}
