use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{delete, put},
    Extension, Json, Router,
};
use rest_types::{SlotTO, VacationPayloadTO};
use service::shiftplan_edit::ShiftplanEditService;
use tracing::instrument;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/slot/:year/:week", put(edit_slot::<RestState>))
        .route(
            "/slot/:slot_id/:year/:week",
            delete(delete_slot::<RestState>),
        )
        .route("/vacation", put(add_vacation::<RestState>))
}

#[instrument(skip(rest_state))]
pub async fn edit_slot<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((year, week)): Path<(u32, u8)>,
    Json(slot): Json<SlotTO>,
) -> Response {
    error_handler(
        (async {
            let slot = SlotTO::from(
                &rest_state
                    .shiftplan_edit_service()
                    .modify_slot(&(&slot).into(), year, week, context.into(), None)
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
pub async fn delete_slot<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((slot_id, year, week)): Path<(Uuid, u32, u8)>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .shiftplan_edit_service()
                .remove_slot(slot_id, year, week, context.into(), None)
                .await?;
            Ok(Response::builder().status(200).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn add_vacation<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(vacation_payload): Json<VacationPayloadTO>,
) -> Response {
    error_handler(
        (async {
            let _ = &rest_state
                .shiftplan_edit_service()
                .add_vacation(
                    vacation_payload.sales_person_id,
                    vacation_payload.from,
                    vacation_payload.to,
                    vacation_payload.description.clone(),
                    context.into(),
                    None,
                )
                .await?;
            Ok(Response::builder().status(200).body(Body::empty()).unwrap())
        })
        .await,
    )
}
