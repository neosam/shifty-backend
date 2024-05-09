use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use service::slot::SlotService;
use uuid::Uuid;

use crate::{error_handler, RestError, RestStateDef};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
impl From<service::slot::DayOfWeek> for DayOfWeek {
    fn from(day_of_week: service::slot::DayOfWeek) -> Self {
        match day_of_week {
            service::slot::DayOfWeek::Monday => Self::Monday,
            service::slot::DayOfWeek::Tuesday => Self::Tuesday,
            service::slot::DayOfWeek::Wednesday => Self::Wednesday,
            service::slot::DayOfWeek::Thursday => Self::Thursday,
            service::slot::DayOfWeek::Friday => Self::Friday,
            service::slot::DayOfWeek::Saturday => Self::Saturday,
            service::slot::DayOfWeek::Sunday => Self::Sunday,
        }
    }
}
impl From<DayOfWeek> for service::slot::DayOfWeek {
    fn from(day_of_week: DayOfWeek) -> Self {
        match day_of_week {
            DayOfWeek::Monday => Self::Monday,
            DayOfWeek::Tuesday => Self::Tuesday,
            DayOfWeek::Wednesday => Self::Wednesday,
            DayOfWeek::Thursday => Self::Thursday,
            DayOfWeek::Friday => Self::Friday,
            DayOfWeek::Saturday => Self::Saturday,
            DayOfWeek::Sunday => Self::Sunday,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotTO {
    #[serde(default)]
    pub id: Uuid,
    pub day_of_week: DayOfWeek,
    pub from: time::Time,
    pub to: time::Time,
    pub valid_from: time::Date,
    pub valid_to: Option<time::Date>,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
impl From<&service::slot::Slot> for SlotTO {
    fn from(slot: &service::slot::Slot) -> Self {
        Self {
            id: slot.id,
            day_of_week: slot.day_of_week.into(),
            from: slot.from,
            to: slot.to,
            valid_from: slot.valid_from,
            valid_to: slot.valid_to,
            deleted: slot.deleted,
            version: slot.version,
        }
    }
}
impl From<&SlotTO> for service::slot::Slot {
    fn from(slot: &SlotTO) -> Self {
        Self {
            id: slot.id,
            day_of_week: slot.day_of_week.into(),
            from: slot.from,
            to: slot.to,
            valid_from: slot.valid_from,
            valid_to: slot.valid_to,
            deleted: slot.deleted,
            version: slot.version,
        }
    }
}

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_all_slots::<RestState>))
        .route("/:id", get(get_slot::<RestState>))
        .route("/", post(create_slot::<RestState>))
        .route("/:id", put(update_slot::<RestState>))
}

pub async fn get_all_slots<RestState: RestStateDef>(rest_state: State<RestState>) -> Response {
    error_handler(
        (async {
            let slots: Arc<[SlotTO]> = rest_state
                .slot_service()
                .get_slots(().into())
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
    Path(slot_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let slot = SlotTO::from(
                &rest_state
                    .slot_service()
                    .get_slot(&slot_id, ().into())
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
    Json(slot): Json<SlotTO>,
) -> Response {
    error_handler(
        (async {
            let slot = SlotTO::from(
                &rest_state
                    .slot_service()
                    .create_slot(&(&slot).into(), ().into())
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
                .update_slot(&(&slot).into(), ().into())
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&slot).unwrap()))
                .unwrap())
        })
        .await,
    )
}
