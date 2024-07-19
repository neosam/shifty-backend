use std::{rc::Rc, sync::Arc};

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{get, post},
    Extension, Json, Router,
};
use rest_types::WorkingHoursTO;

use service::working_hours::WorkingHoursService;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route(
            "/for-week/:sales_person_id/:year/:calendar_week",
            get(get_working_hours_for_week::<RestState>),
        )
        .route("/", post(create_working_hours::<RestState>))
}

pub async fn create_working_hours<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(working_hours): Json<WorkingHoursTO>,
) -> Response {
    error_handler(
        (async {
            let working_hours = WorkingHoursTO::from(
                &rest_state
                    .working_hours_service()
                    .create(&(&working_hours).into(), context.into())
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&working_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn get_working_hours_for_week<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((sales_person_id, year, calendar_week)): Path<(Uuid, u32, u8)>,
) -> Response {
    error_handler(
        (async {
            let working_hours = WorkingHoursTO::from(
                &rest_state
                    .working_hours_service()
                    .find_for_week(sales_person_id, calendar_week, year, context.into())
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&working_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}
