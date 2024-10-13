use std::sync::Arc;

use axum::body::Body;
use axum::extract::Path;
use axum::routing::get;
use axum::{extract::State, response::Response};
use axum::{Extension, Router};
use rest_types::BookingConflictTO;
use service::booking_information::BookingInformationService;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route(
        "/conflicts/for-week/:year/:week",
        get(get_booking_conflicts_for_week::<RestState>),
    )
}

pub async fn get_booking_conflicts_for_week<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((year, week)): Path<(u32, u8)>,
) -> Response {
    error_handler(
        (async {
            let booking_conflicts: Arc<[BookingConflictTO]> = rest_state
                .booking_information_service()
                .get_booking_conflicts_for_week(year, week, context.into())
                .await?
                .iter()
                .map(BookingConflictTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(
                    serde_json::to_string(&booking_conflicts).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}
