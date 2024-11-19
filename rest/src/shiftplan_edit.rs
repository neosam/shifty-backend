use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::put,
    Extension, Json, Router,
};
use rest_types::SlotTO;
use service::shiftplan_edit::ShiftplanEditService;
use tracing::instrument;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/slot/:year/:week", put(edit_slot::<RestState>))
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
                    .modify_slot(&(&slot).into(), year, week, context.into())
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
