//! Phase 55 Plan 02 (F3 REB-MANUAL + F5 HR-ALERT) REST-Layer.
//!
//! Vier neue Routen — der REST-Layer ist ein duenner Wrapper ueber die BL:
//!
//! - `POST /rebooking/manual` — HR bucht manuell um (REB-MANUAL-01).
//! - `GET /rebooking-suggestions` — HR sieht alle offenen HrSuggestion-
//!   Batches (HR-ALERT-02).
//! - `POST /rebooking-suggestions/{id}/approve` — Pending → Approved +
//!   Pair-ExtraHours (HR-ALERT-03).
//! - `POST /rebooking-suggestions/{id}/reject` — Pending → Rejected ohne
//!   Pair-Rows (HR-ALERT-03).
//!
//! HR-Gate: von der BL erledigt. REST leitet den Auth-Context 1:1 durch.
//! Kein Undo/Delete-Endpoint (D-55-04).
//!
//! Error-Mapping:
//! - `EntityAlreadyExists` (UNIQUE-Slot-Kollision) → HTTP 409 +
//!   `{"error":"RebookingErrorSlotTaken"}` (T-4: kein SQL-Leak, i18n-Key
//!   fuer FE).
//! - `BatchAlreadyResolved` (Race auf approve/reject) → HTTP 409 +
//!   `{"error":"RebookingErrorAlreadyResolved"}`.
//! - Sonstige `ServiceError` → globales `error_handler`-Mapping.

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{get, post},
    Extension, Json, Router,
};
use rest_types::{ManualRebookingRequestTO, RebookingBatchTO, RebookingSuggestionTO};
use service::rebooking_reconciliation::{RebookingDirection, RebookingReconciliationService};
use service::ServiceError;
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

/// Router-Segment fuer `/rebooking/*` (nur `POST /manual`).
pub fn generate_manual_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/manual", post(post_manual::<RestState>))
}

/// Router-Segment fuer `/rebooking-suggestions/*` (GET pending +
/// approve/reject).
pub fn generate_suggestions_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_pending::<RestState>))
        .route("/{id}/approve", post(post_approve::<RestState>))
        .route("/{id}/reject", post(post_reject::<RestState>))
}

fn conflict_body(i18n_key: &str) -> Response {
    // Manual JSON-Body — kein serde_json::to_string, damit der i18n-Key
    // stabil ist (kein Escaping-Ueberraschungen) und der Body deterministisch
    // bleibt (T-4 Mitigation: kein SQL-Leak / kein Batch-id-Leak).
    let body = format!("{{\"error\":\"{}\"}}", i18n_key);
    Response::builder()
        .status(409)
        .header("Content-Type", "application/json")
        .body(Body::new(body))
        .unwrap()
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/manual",
    tags = ["Rebooking"],
    request_body = ManualRebookingRequestTO,
    responses(
        (status = 200, description = "Manual rebooking accepted; returns the new batch", body = RebookingBatchTO, content_type = "application/json"),
        (status = 400, description = "Invalid request (hours <= 0, invalid iso_week etc.) — REB-MANUAL-03"),
        (status = 403, description = "Forbidden — HR role required"),
        (status = 409, description = "Slot already taken (UNIQUE (sp, iso_year, iso_week) collision)", content_type = "application/json"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_manual<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(payload): Json<ManualRebookingRequestTO>,
) -> Response {
    // REB-MANUAL-03: hours <= 0.0 → 400 mit strukturiertem Body.
    if !(payload.hours.is_finite()) || payload.hours <= 0.0 {
        return Response::builder()
            .status(400)
            .header("Content-Type", "application/json")
            .body(Body::new(
                "{\"error\":\"RebookingErrorHoursMustBePositive\"}".to_string(),
            ))
            .unwrap();
    }
    // D-55-06: direction als eigenes Feld, nicht aus hours-Vorzeichen abgeleitet.
    let direction: RebookingDirection = (&payload.direction).into();
    let result = rest_state
        .rebooking_reconciliation_service()
        .rebook_manual(
            payload.sales_person_id,
            payload.iso_year,
            payload.iso_week,
            direction,
            payload.hours,
            context.into(),
            None,
        )
        .await;
    match result {
        Ok(entity) => {
            let to = RebookingBatchTO::from(&entity);
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
                .unwrap()
        }
        // T-4 Mitigation: UNIQUE-Kollision liefert einen strukturierten
        // Fehler-Body statt der rohen `EntityAlreadyExists(id)`-Meldung; das
        // FE branched auf `error == "RebookingErrorSlotTaken"`.
        Err(ServiceError::EntityAlreadyExists(_)) => conflict_body("RebookingErrorSlotTaken"),
        Err(err) => error_handler(Err(err.into())),
    }
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "",
    tags = ["Rebooking"],
    responses(
        (status = 200, description = "All pending HR-suggestion batches (phase-wide, HR-ALERT-02)", body = [RebookingSuggestionTO], content_type = "application/json"),
        (status = 403, description = "Forbidden — HR role required"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_pending<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            // Plan 55-01: `list_pending_for_sales_person(None, ...)` liefert
            // phase-weit (alle SP mit einem pending HrSuggestion). Die BL
            // hydriert intern zu `RebookingSuggestion` inkl. IST/DANN.
            let suggestions = rest_state
                .rebooking_reconciliation_service()
                .list_pending_for_sales_person(None, context.into(), None)
                .await?;
            let payload: Vec<RebookingSuggestionTO> =
                suggestions.iter().map(RebookingSuggestionTO::from).collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&payload).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/{id}/approve",
    tags = ["Rebooking"],
    params(
        ("id" = Uuid, Path, description = "Rebooking batch id (kind=HrSuggestion, state=Pending)")
    ),
    responses(
        (status = 200, description = "Batch approved; state=Approved and Pair-ExtraHours written", body = RebookingBatchTO, content_type = "application/json"),
        (status = 403, description = "Forbidden — HR role required"),
        (status = 409, description = "Race — batch already resolved (parallel HR action)", content_type = "application/json"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_approve<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Path(batch_id): Path<Uuid>,
    Extension(context): Extension<Context>,
) -> Response {
    let result = rest_state
        .rebooking_reconciliation_service()
        .approve_suggestion(batch_id, context.into(), None)
        .await;
    match result {
        Ok(entity) => {
            let to = RebookingBatchTO::from(&entity);
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
                .unwrap()
        }
        Err(ServiceError::BatchAlreadyResolved) => {
            conflict_body("RebookingErrorAlreadyResolved")
        }
        Err(err) => error_handler(Err(err.into())),
    }
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/{id}/reject",
    tags = ["Rebooking"],
    params(
        ("id" = Uuid, Path, description = "Rebooking batch id (kind=HrSuggestion, state=Pending)")
    ),
    responses(
        (status = 200, description = "Batch rejected; state=Rejected, no Pair-ExtraHours written (D-55-07)", body = RebookingBatchTO, content_type = "application/json"),
        (status = 403, description = "Forbidden — HR role required"),
        (status = 409, description = "Race — batch already resolved", content_type = "application/json"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_reject<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Path(batch_id): Path<Uuid>,
    Extension(context): Extension<Context>,
) -> Response {
    let result = rest_state
        .rebooking_reconciliation_service()
        .reject_suggestion(batch_id, context.into(), None)
        .await;
    match result {
        Ok(entity) => {
            let to = RebookingBatchTO::from(&entity);
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
                .unwrap()
        }
        Err(ServiceError::BatchAlreadyResolved) => {
            conflict_body("RebookingErrorAlreadyResolved")
        }
        Err(err) => error_handler(Err(err.into())),
    }
}

/// OpenAPI-Sicht auf `/rebooking/manual`. Nested unter `/rebooking` in
/// `rest/src/lib.rs::ApiDoc`.
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Rebooking", description = "Manual rebooking (F3)")
    ),
    paths(post_manual),
    components(schemas(
        ManualRebookingRequestTO,
        RebookingBatchTO,
        rest_types::RebookingBatchKindTO,
        rest_types::RebookingBatchStateTO,
        rest_types::RebookingDirectionTO,
    ))
)]
pub struct RebookingManualApiDoc;

/// OpenAPI-Sicht auf `/rebooking-suggestions/*`. Nested unter
/// `/rebooking-suggestions` in `rest/src/lib.rs::ApiDoc`.
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Rebooking", description = "HR-alert suggestions (F5)")
    ),
    paths(get_pending, post_approve, post_reject),
    components(schemas(
        RebookingBatchTO,
        RebookingSuggestionTO,
        rest_types::RebookingBatchKindTO,
        rest_types::RebookingBatchStateTO,
        rest_types::RebookingDirectionTO,
    ))
)]
pub struct RebookingSuggestionsApiDoc;
