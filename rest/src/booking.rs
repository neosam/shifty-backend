use std::sync::Arc;

use axum::body::Body;
use axum::extract::Path;
use axum::routing::{delete, get, post};
use axum::{extract::State, response::Response};
use axum::{Extension, Json, Router};
use rest_types::BookingTO;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};
use service::booking::{Booking, BookingService};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_all_bookings::<RestState>))
        .route("/week/:year/:calendar_week", get(get_by_week::<RestState>))
        .route("/:id", get(get_booking::<RestState>))
        .route("/", post(create_booking::<RestState>))
        .route("/:id", delete(delete_booking::<RestState>))
}

pub async fn get_all_bookings<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let bookings: Arc<[BookingTO]> = rest_state
                .booking_service()
                .get_all(context.into())
                .await?
                .iter()
                .map(BookingTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&bookings).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn get_by_week<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((year, calendar_week)): Path<(u32, u8)>,
) -> Response {
    error_handler(
        (async {
            let bookings: Arc<[BookingTO]> = rest_state
                .booking_service()
                .get_for_week(calendar_week, year, context.into())
                .await?
                .iter()
                .map(BookingTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&bookings).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn get_booking<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(booking_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let booking = rest_state
                .booking_service()
                .get(booking_id, context.into())
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::new(
                    serde_json::to_string(&BookingTO::from(&booking)).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

pub async fn create_booking<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(booking): Json<BookingTO>,
) -> Response {
    error_handler(
        (async {
            let booking = rest_state
                .booking_service()
                .create(&Booking::from(&booking), context.into())
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::new(
                    serde_json::to_string(&BookingTO::from(&booking)).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

pub async fn delete_booking<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(booking_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .booking_service()
                .delete(booking_id, context.into())
                .await?;
            Ok(Response::builder().status(200).body(Body::empty()).unwrap())
        })
        .await,
    )
}
