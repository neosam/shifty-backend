//! REST-Layer fuer die Rebooking-Reconciliation-Domain (Phase 55, 55-03).
//!
//! Route-Set D-55-B (Modell B), aufgeteilt in zwei Nests:
//! - Nest `/rebooking`:
//!   - POST `/manual` → `create_manual_rebooking` (F3, one-shot)
//! - Nest `/rebooking-suggestions`:
//!   - POST `/` → `create_rebooking_suggestion` (F5-Suggest/Claim — bewusste
//!     Mutation, erzeugt den pending-Batch und liefert die IST/DANN-Vorschau)
//!   - POST `/{id}/approve` → `approve_rebooking_suggestion`
//!   - POST `/{id}/reject` → `reject_rebooking_suggestion`
//!
//! Alle Handler sind duenne Wrapper: sie mappen DTO ↔ Domain, dispatchen ueber
//! `rest_state.rebooking_reconciliation_service()` und delegieren das
//! Status-Mapping (403/409/422) AUSSCHLIESSLICH an den zentralen
//! `error_handler` (kein manuelles Status-Mapping im Handler). Die HR-
//! Autorisierung liegt im Service (55-02, T-55-10); UNIQUE-Slot-Kollision und
//! Double-Approve-Race werden vom Service als `EntityAlreadyExists`
//! signalisiert und zentral → HTTP 409.

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::post,
    Extension, Json, Router,
};
use rest_types::rebooking::{
    ManualRebookingRequestTO, RebookingApproveRequestTO, RebookingSuggestRequestTO,
    RebookingSuggestionTO,
};
use service::rebooking_reconciliation::RebookingReconciliationService;
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

/// Router fuer den `/rebooking`-Nest (F3 one-shot).
pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/manual", post(create_manual_rebooking::<RestState>))
}

/// Router fuer den `/rebooking-suggestions`-Nest (F5-Modell-B Lifecycle).
pub fn generate_suggestions_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", post(create_rebooking_suggestion::<RestState>))
        .route("/{id}/approve", post(approve_rebooking_suggestion::<RestState>))
        .route("/{id}/reject", post(reject_rebooking_suggestion::<RestState>))
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/manual",
    tags = ["Rebooking"],
    request_body = ManualRebookingRequestTO,
    responses(
        (status = 204, description = "Manuelle Umbuchung durchgefuehrt (F3 one-shot)"),
        (status = 403, description = "Forbidden (nicht HR)"),
        (status = 409, description = "UNIQUE-Slot-Kollision (Woche bereits belegt)"),
        (status = 422, description = "Validierungsfehler"),
    ),
)]
pub async fn create_manual_rebooking<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(body): Json<ManualRebookingRequestTO>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .rebooking_reconciliation_service()
                .rebook_manual(
                    body.sales_person_id,
                    body.direction.into(),
                    body.hours,
                    body.iso_year,
                    body.iso_week,
                    context.into(),
                    None,
                )
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["Rebooking"],
    request_body = RebookingSuggestRequestTO,
    responses(
        (status = 200, description = "Pending-Batch geclaimt (idempotent); backend-computed IST/DANN-Vorschau", body = RebookingSuggestionTO),
        (status = 403, description = "Forbidden (nicht HR)"),
        (status = 409, description = "Slot bereits entschieden (approved|rejected)"),
        (status = 422, description = "Validierungsfehler"),
    ),
)]
pub async fn create_rebooking_suggestion<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(body): Json<RebookingSuggestRequestTO>,
) -> Response {
    error_handler(
        (async {
            let suggestion = rest_state
                .rebooking_reconciliation_service()
                .suggest(
                    body.sales_person_id,
                    body.iso_year,
                    body.iso_week,
                    context.into(),
                    None,
                )
                .await?;
            let to = RebookingSuggestionTO::from_domain(&suggestion, body.sales_person_id);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
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
    params(("id", description = "Rebooking-Batch id (pending)")),
    request_body = RebookingApproveRequestTO,
    responses(
        (status = 204, description = "Vorschlag genehmigt; -N/+N-Pair geschrieben"),
        (status = 403, description = "Forbidden (nicht HR)"),
        (status = 409, description = "Double-Approve-Race-Verlierer (bereits entschieden)"),
        (status = 422, description = "Validierungsfehler"),
    ),
)]
pub async fn approve_rebooking_suggestion<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
    Json(body): Json<RebookingApproveRequestTO>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .rebooking_reconciliation_service()
                .approve(id, body.hours, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/{id}/reject",
    tags = ["Rebooking"],
    params(("id", description = "Rebooking-Batch id (pending)")),
    responses(
        (status = 204, description = "Vorschlag abgelehnt; kein Pair"),
        (status = 403, description = "Forbidden (nicht HR)"),
        (status = 409, description = "bereits entschieden"),
    ),
)]
pub async fn reject_rebooking_suggestion<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .rebooking_reconciliation_service()
                .reject(id, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        create_manual_rebooking,
        create_rebooking_suggestion,
        approve_rebooking_suggestion,
        reject_rebooking_suggestion,
    ),
    components(schemas(
        ManualRebookingRequestTO,
        RebookingSuggestRequestTO,
        RebookingApproveRequestTO,
        RebookingSuggestionTO,
        rest_types::rebooking::RebookingDirectionTO,
    )),
    tags(
        (name = "Rebooking", description = "Manuelle Umbuchung (F3) + HR-Alert-Vorschlaege (F5, Modell B)"),
    ),
)]
pub struct RebookingApiDoc;
