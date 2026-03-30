use axum::{
    extract::{Path, State},
    routing::get,
    Extension, Router,
};
use utoipa::OpenApi;

use crate::{error_handler, Context, Response, RestStateDef};
use rest_types::ShiftplanWeekTO;
use service::{permission::Authentication, shiftplan::ShiftplanViewService};
use uuid::Uuid;

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/{shiftplan_id}/{year}/{week}", get(get_shiftplan_week::<RestState>))
}

#[utoipa::path(
    get,
    path = "/{shiftplan_id}/{year}/{week}",
    params(
        ("shiftplan_id" = Uuid, Path, description = "Shift plan ID"),
        ("year" = u32, Path, description = "Year of the shift plan"),
        ("week" = u8, Path, description = "Calendar week number (1-53)")
    ),
    responses(
        (status = 200, description = "Shift plan for the specified week", body = ShiftplanWeekTO),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "shiftplan"
)]
async fn get_shiftplan_week<RestState: RestStateDef>(
    Path((shiftplan_id, year, week)): Path<(Uuid, u32, u8)>,
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        async {
            let shiftplan = rest_state
                .shiftplan_view_service()
                .get_shiftplan_week(shiftplan_id, year, week, Authentication::Context(context), None)
                .await?;

            let shiftplan_to = ShiftplanWeekTO::from(&shiftplan);

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&shiftplan_to).unwrap(),
                ))
                .unwrap())
        }
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_shiftplan_week,
    ),
    components(
        schemas(
            ShiftplanWeekTO,
        )
    ),
    tags(
        (name = "shiftplan", description = "Shift plan management")
    )
)]
pub struct ShiftplanApiDoc;
