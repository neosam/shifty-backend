//! REST-Layer fuer die Absence-Domain (Phase 1 — Range-based absence).
//!
//! Sechs Routen unter `/absence-period` (Bindestrich, D-01):
//! POST `/`, GET `/`, GET `/{id}`, PUT `/{id}`, DELETE `/{id}`,
//! GET `/by-sales-person/{sales_person_id}`. Jeder Handler traegt
//! `#[utoipa::path]` (CC-06) + `#[instrument(skip(rest_state))]`.
//!
//! PUT-Handler ueberschreibt `entity.id = path_id` (path-id wins). Die
//! Service-Layer-Verifikation (Permission, Self-Overlap, Range) wird in
//! `service::absence::AbsenceService` durchgefuehrt; der REST-Layer ist
//! ein duenner Wrapper mit DTO-Conversion und Error-Mapping via
//! `error_handler`. Alle Handler dispatchen ueber `rest_state.absence_service()`
//! gemaess `RestStateDef`-Trait.

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use rest_types::{AbsenceCategoryTO, AbsencePeriodTO};
use service::absence::AbsenceService;
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", post(create_absence_period::<RestState>))
        .route("/", get(get_all_absence_periods::<RestState>))
        .route("/{id}", get(get_absence_period::<RestState>))
        .route("/{id}", put(update_absence_period::<RestState>))
        .route("/{id}", delete(delete_absence_period::<RestState>))
        .route(
            "/by-sales-person/{sales_person_id}",
            get(get_absence_periods_for_sales_person::<RestState>),
        )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["Absence"],
    request_body = AbsencePeriodTO,
    responses(
        (status = 201, description = "Absence period created", body = AbsencePeriodTO),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Validation error"),
    ),
)]
pub async fn create_absence_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(body): Json<AbsencePeriodTO>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            // Phase-3-Plan-03 minimaler Diff: Wrapper-Result wird unwrappt
            // und nur `.absence` in den Body gemappt — Warnings werden
            // dropped. Plan 05 ergänzt `AbsencePeriodCreateResultTO` und
            // dreht die Body-Form um.
            // TODO Plan-05: AbsencePeriodCreateResultTO statt AbsencePeriodTO im Body.
            let result = svc.create(&(&body).into(), context.into(), None).await?;
            let entity = AbsencePeriodTO::from(&result.absence);
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&entity).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "",
    tags = ["Absence"],
    responses(
        (status = 200, description = "All absence periods", body = [AbsencePeriodTO]),
        (status = 403, description = "Forbidden"),
    ),
)]
pub async fn get_all_absence_periods<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            let entities = svc.find_all(context.into(), None).await?;
            let tos: Vec<AbsencePeriodTO> = entities.iter().map(AbsencePeriodTO::from).collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&tos).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}",
    tags = ["Absence"],
    params(("id", description = "Absence period logical id")),
    responses(
        (status = 200, description = "Absence period", body = AbsencePeriodTO),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
)]
pub async fn get_absence_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            let entity = svc.find_by_id(id, context.into(), None).await?;
            let to = AbsencePeriodTO::from(&entity);
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
    put,
    path = "/{id}",
    tags = ["Absence"],
    params(("id", description = "Absence period logical id")),
    request_body = AbsencePeriodTO,
    responses(
        (status = 200, description = "Updated absence period", body = AbsencePeriodTO),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Version conflict"),
        (status = 422, description = "Validation error"),
    ),
)]
pub async fn update_absence_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(absence_id): Path<Uuid>,
    Json(body): Json<AbsencePeriodTO>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            let mut entity: service::absence::AbsencePeriod = (&body).into();
            entity.id = absence_id; // path-id wins (D-01 / Pitfall guard)
            // TODO Plan-05: AbsencePeriodCreateResultTO statt AbsencePeriodTO im Body.
            let result = svc.update(&entity, context.into(), None).await?;
            let updated = AbsencePeriodTO::from(&result.absence);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&updated).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/{id}",
    tags = ["Absence"],
    params(("id", description = "Absence period logical id")),
    responses(
        (status = 204, description = "Soft-deleted"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
)]
pub async fn delete_absence_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            svc.delete(id, context.into(), None).await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/by-sales-person/{sales_person_id}",
    tags = ["Absence"],
    params(("sales_person_id", description = "Sales person id")),
    responses(
        (status = 200, description = "Absence periods for sales person", body = [AbsencePeriodTO]),
        (status = 403, description = "Forbidden"),
    ),
)]
pub async fn get_absence_periods_for_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            let entities = svc
                .find_by_sales_person(sales_person_id, context.into(), None)
                .await?;
            let tos: Vec<AbsencePeriodTO> = entities.iter().map(AbsencePeriodTO::from).collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&tos).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        create_absence_period,
        get_all_absence_periods,
        get_absence_period,
        update_absence_period,
        delete_absence_period,
        get_absence_periods_for_sales_person,
    ),
    components(schemas(AbsencePeriodTO, AbsenceCategoryTO)),
    tags(
        (name = "Absence", description = "Absence period management (range-based)"),
    ),
)]
pub struct AbsenceApiDoc;
