use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query};
use axum::routing::{delete, get, post};
use axum::{extract::State, response::Response};
use axum::{Extension, Json, Router};
use rest_types::BookingTO;
use tracing::instrument;
use uuid::Uuid;

use crate::{error_handler, Context, RestError, RestStateDef};
use service::booking::{Booking, BookingService};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_all_bookings::<RestState>))
        .route("/week/:year/:calendar_week", get(get_by_week::<RestState>))
        .route("/:id", get(get_booking::<RestState>))
        .route("/", post(create_booking::<RestState>))
        .route("/:id", delete(delete_booking::<RestState>))
        .route("/copy", post(copy_calendar_week::<RestState>))
}

#[instrument(skip(rest_state))]
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

#[instrument(skip(rest_state))]
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

#[instrument(skip(rest_state))]
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

#[instrument(skip(rest_state))]
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

#[instrument(skip(rest_state))]
pub async fn copy_calendar_week<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    error_handler(
        (async {
            let from_year = params
                .get("from_year")
                .ok_or_else(|| RestError::BadRequest("year missing".to_string()))?
                .parse()?;
            let from_week = params
                .get("from_week")
                .ok_or_else(|| RestError::BadRequest("week missing".to_string()))?
                .parse()?;
            let to_year = params
                .get("to_year")
                .ok_or_else(|| RestError::BadRequest("year missing".to_string()))?
                .parse()?;
            let to_week = params
                .get("to_week")
                .ok_or_else(|| RestError::BadRequest("week missing".to_string()))?
                .parse()?;
            rest_state
                .booking_service()
                .copy_week(from_week, from_year, to_week, to_year, context.into())
                .await?;
            Ok(Response::builder().status(200).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
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
