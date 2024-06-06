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
use uuid::Uuid;

use crate::{error_handler, Context, RestError, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_all_slots::<RestState>))
        .route("/:id", get(get_slot::<RestState>))
        .route("/", post(create_slot::<RestState>))
        .route("/:id", put(update_slot::<RestState>))
}

pub async fn get_all_slots<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let slots: Arc<[SlotTO]> = rest_state
                .slot_service()
                .get_slots(context.into())
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
                    .get_slot(&slot_id, context.into())
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
                    .create_slot(&(&slot).into(), context.into())
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
                .update_slot(&(&slot).into(), context.into())
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&slot).unwrap()))
                .unwrap())
        })
        .await,
    )
}
