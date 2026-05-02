use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{delete, post, put},
    Extension, Json, Router,
};
use rest_types::{
    AbsenceCategoryTO, BookingCreateResultTO, BookingTO, CopyWeekResultTO, DayOfWeekTO, SlotTO,
    VacationPayloadTO, WarningTO,
};
use serde::{Deserialize, Serialize};
use service::shiftplan_edit::ShiftplanEditService;
use tracing::instrument;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/slot/{year}/{week}", put(edit_slot::<RestState>))
        .route(
            "/slot/{slot_id}/{year}/{week}",
            delete(delete_slot::<RestState>),
        )
        .route("/vacation", put(add_vacation::<RestState>))
        // Phase-3 (C-Phase3-09) — konflikt-aware Booking-Endpunkte. Die
        // existierenden `POST /booking` und `POST /booking/copy-week`
        // unter dem `booking` Router bleiben unverändert (D-Phase3-18
        // Regression-Lock).
        .route(
            "/booking",
            post(book_slot_with_conflict_check::<RestState>),
        )
        .route(
            "/copy-week",
            post(copy_week_with_conflict_check::<RestState>),
        )
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

/// Request-Body für `POST /shiftplan-edit/copy-week`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CopyWeekRequest {
    pub from_year: u32,
    pub from_calendar_week: u8,
    pub to_year: u32,
    pub to_calendar_week: u8,
}

/// Phase 3 — konflikt-aware Booking-Persist (C-Phase3-09).
///
/// Persistiert das Booking via `ShiftplanEditService::book_slot_with_conflict_check`.
/// Liefert das Booking + alle Cross-Source-Warnings (BOOK-02 Reverse-Warning).
#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/booking",
    tags = ["ShiftplanEdit"],
    request_body = BookingTO,
    responses(
        (status = 201, description = "Booking created (with cross-source warnings if any)", body = BookingCreateResultTO),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Validation error"),
    ),
)]
pub async fn book_slot_with_conflict_check<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(body): Json<BookingTO>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.shiftplan_edit_service();
            let booking: service::booking::Booking = (&body).into();
            let result = svc
                .book_slot_with_conflict_check(&booking, context.into(), None)
                .await?;
            let to = BookingCreateResultTO::from(&result);
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
                .unwrap())
        })
        .await,
    )
}

/// Phase 3 — konflikt-aware copy_week (D-Phase3-02).
///
/// Iteriert über die Bookings der Quell-Woche, ruft pro Source-Booking
/// intern `book_slot_with_conflict_check` und aggregiert ALLE Warnings
/// (D-Phase3-15: KEINE De-Dup). Permission `shiftplan.edit` (Bulk-Op).
#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/copy-week",
    tags = ["ShiftplanEdit"],
    request_body = CopyWeekRequest,
    responses(
        (status = 200, description = "Bookings copied (with cross-source warnings if any)", body = CopyWeekResultTO),
        (status = 403, description = "Forbidden"),
    ),
)]
pub async fn copy_week_with_conflict_check<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(req): Json<CopyWeekRequest>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.shiftplan_edit_service();
            let result = svc
                .copy_week_with_conflict_check(
                    req.from_calendar_week,
                    req.from_year,
                    req.to_calendar_week,
                    req.to_year,
                    context.into(),
                    None,
                )
                .await?;
            let to = CopyWeekResultTO::from(&result);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
                .unwrap())
        })
        .await,
    )
}

/// OpenAPI-Bündel für die Phase-3-Endpunkte unter `/shiftplan-edit`.
///
/// Die existierenden `edit_slot`/`delete_slot`/`add_vacation`-Handler
/// haben heute (Phase 1) keine `#[utoipa::path]`-Annotation und werden
/// daher hier NICHT in `paths(...)` aufgenommen — utoipa würde sonst
/// einen Compile-Error werfen. Phase-3 ergänzt nur die zwei neuen
/// konflikt-aware Endpunkte.
#[derive(OpenApi)]
#[openapi(
    paths(
        book_slot_with_conflict_check,
        copy_week_with_conflict_check,
    ),
    components(schemas(
        BookingTO,
        BookingCreateResultTO,
        CopyWeekResultTO,
        CopyWeekRequest,
        WarningTO,
        AbsenceCategoryTO,
        DayOfWeekTO,
    )),
    tags(
        (name = "ShiftplanEdit", description = "Shiftplan edit operations (conflict-aware)"),
    ),
)]
pub struct ShiftplanEditApiDoc;
