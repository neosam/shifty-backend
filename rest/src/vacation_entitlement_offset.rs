//! REST-Layer für den Vacation-Entitlement-Offset (Phase 28 — VAC-OFFSET-01).
//!
//! HR-gated CRUD unter `/vacation-entitlement-offset`:
//! - `POST /` — Upsert (set) des signierten Offsets für (sales_person_id, year).
//! - `DELETE /{sales_person_id}/{year}` — Löschen (soft-delete) des Offsets.
//!
//! Year-scoped (eine aktive Zeile pro Person+Jahr, D-28-09). Die
//! HR-Privilege-Enforcement passiert AUSSCHLIESSLICH im Basic-Service
//! (`service_impl::vacation_entitlement_offset`, D-28-06b); der REST-Layer ist
//! ein dünner Wrapper mit DTO-Conversion und Error-Mapping via `error_handler`.
//! Jeder Handler trägt `#[utoipa::path]` (CC-06) + `#[instrument(skip(rest_state))]`.

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{delete, post},
    Extension, Json, Router,
};
use rest_types::VacationEntitlementOffsetTO;
use service::vacation_entitlement_offset::VacationEntitlementOffsetService;
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", post(set_vacation_entitlement_offset::<RestState>))
        .route(
            "/{sales_person_id}/{year}",
            delete(delete_vacation_entitlement_offset::<RestState>),
        )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["VacationEntitlementOffset"],
    request_body = VacationEntitlementOffsetTO,
    responses(
        (status = 200, description = "Offset set (upsert)", body = VacationEntitlementOffsetTO, content_type = "application/json"),
        (status = 403, description = "Forbidden — HR privilege required"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn set_vacation_entitlement_offset<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(offset): Json<VacationEntitlementOffsetTO>,
) -> Response {
    error_handler(
        (async {
            let result = rest_state
                .vacation_entitlement_offset_service()
                .set(
                    offset.sales_person_id,
                    offset.year,
                    offset.offset_days,
                    context.into(),
                    None,
                )
                .await?;
            let to = VacationEntitlementOffsetTO {
                sales_person_id: result.sales_person_id,
                year: result.year,
                offset_days: result.offset_days,
            };
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
    delete,
    path = "/{sales_person_id}/{year}",
    tags = ["VacationEntitlementOffset"],
    params(
        ("sales_person_id" = Uuid, Path, description = "Sales person ID"),
        ("year" = u32, Path, description = "Calendar year (e.g. 2026)"),
    ),
    responses(
        (status = 204, description = "Offset deleted"),
        (status = 403, description = "Forbidden — HR privilege required"),
        (status = 404, description = "No offset set for this person/year"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn delete_vacation_entitlement_offset<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((sales_person_id, year)): Path<(Uuid, u32)>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .vacation_entitlement_offset_service()
                .delete(sales_person_id, year, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (
            name = "VacationEntitlementOffset",
            description = "HR-gated signed vacation-entitlement offset (per person + year)",
        ),
    ),
    paths(
        set_vacation_entitlement_offset,
        delete_vacation_entitlement_offset,
    ),
    components(schemas(VacationEntitlementOffsetTO)),
)]
pub struct VacationEntitlementOffsetApiDoc;
