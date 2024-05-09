use std::sync::Arc;

use axum::body::Body;
use axum::extract::Path;
use axum::routing::{delete, get, post};
use axum::{extract::State, response::Response};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::{error_handler, RestStateDef};
use service::booking::{BookingService, Booking};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BookingTO {
    #[serde(default)]
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub slot_id: Uuid,
    pub calendar_week: i32,
    pub year: u32,
    #[serde(default)]
    pub created: Option<PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
impl From<&Booking> for BookingTO {
    fn from(booking: &Booking) -> Self {
        Self {
            id: booking.id,
            sales_person_id: booking.sales_person_id,
            slot_id: booking.slot_id,
            calendar_week: booking.calendar_week,
            year: booking.year,
            created: booking.created,
            deleted: booking.deleted,
            version: booking.version,
        }
    }
}
impl From<&BookingTO> for Booking {
    fn from(booking: &BookingTO) -> Self {
        Self {
            id: booking.id,
            sales_person_id: booking.sales_person_id,
            slot_id: booking.slot_id,
            calendar_week: booking.calendar_week,
            year: booking.year,
            created: booking.created,
            deleted: booking.deleted,
            version: booking.version,
        }
    }
}

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_all_bookings::<RestState>))
        .route("/:id", get(get_booking::<RestState>))
        .route("/", post(create_booking::<RestState>))
        .route("/:id", delete(delete_booking::<RestState>))
}

pub async fn get_all_bookings<RestState: RestStateDef>(
    rest_state: State<RestState>,
) -> Response {
    error_handler(
        (async {
            let bookings: Arc<[BookingTO]> = rest_state
                .booking_service()
                .get_all(())
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
    Path(booking_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let booking = rest_state.booking_service().get(booking_id, ()).await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&BookingTO::from(&booking)).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn create_booking<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Json(booking): Json<BookingTO>,
) -> Response {
    error_handler(
        (async {
            let booking = rest_state.booking_service().create(&Booking::from(&booking), ()).await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&BookingTO::from(&booking)).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn delete_booking<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Path(booking_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state.booking_service().delete(booking_id, ()).await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::empty())
                .unwrap())
        })
        .await,
    )
}