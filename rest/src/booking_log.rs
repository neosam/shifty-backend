use axum::{
    body::Body,
    extract::{Path, State},
    routing::get,
    Extension, Router,
};
use tracing::instrument;
use utoipa::OpenApi;

use crate::{error_handler, Context, Response, RestStateDef};
use rest_types::BookingLogTO;
use service::booking_log::BookingLogService;
use service::permission::Authentication;

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/{year}/{week}", get(get_booking_logs_for_week::<RestState>))
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{year}/{week}",
    params(
        ("year" = u32, Path, description = "Year of the booking logs"),
        ("week" = u8, Path, description = "Calendar week number (1-53)")
    ),
    responses(
        (status = 200, description = "List of booking logs for the specified week", body = Vec<BookingLogTO>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - Requires SHIFTPLANNER privilege"),
        (status = 500, description = "Internal server error")
    ),
    tag = "booking_log"
)]
async fn get_booking_logs_for_week<RestState: RestStateDef>(
    Path((year, week)): Path<(u32, u8)>,
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        async {
            let logs = rest_state
                .booking_log_service()
                .get_booking_logs_for_week(year, week, Authentication::Context(context), None)
                .await?;

            let logs_to: Vec<BookingLogTO> = logs.iter().map(BookingLogTO::from).collect();

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&logs_to).unwrap()))
                .unwrap())
        }
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_booking_logs_for_week,
    ),
    components(
        schemas(
            BookingLogTO,
        )
    ),
    tags(
        (name = "booking_log", description = "Booking log management - provides read-only access to booking audit trail")
    )
)]
pub struct BookingLogApiDoc;
