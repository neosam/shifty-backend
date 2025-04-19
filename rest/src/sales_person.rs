use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, Query};
use axum::routing::{delete, get, post, put};
use axum::{extract::State, response::Response};
use axum::{Extension, Json, Router};
use rest_types::{SalesPersonTO, SalesPersonUnavailableTO};
use serde::Deserialize;
use service::block::BlockService;
use service::sales_person::SalesPersonService;
use service::sales_person_unavailable::SalesPersonUnavailableService;
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestError, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_all_sales_persons::<RestState>))
        .route("/{id}", get(get_sales_person::<RestState>))
        .route("/{id}/ical", get(ical_for_sales_person::<RestState>))
        .route("/", post(create_sales_person::<RestState>))
        .route("/{id}", put(update_sales_person::<RestState>))
        .route("/{id}", delete(delete_sales_person::<RestState>))
        .route("/{id}/user", get(get_sales_person_user::<RestState>))
        .route("/{id}/user", post(set_sales_person_user::<RestState>))
        .route("/{id}/user", delete(delete_sales_person_user::<RestState>))
        .route(
            "/{id}/unavailable",
            get(get_sales_person_unavailable::<RestState>),
        )
        .route(
            "/unavailable",
            post(create_sales_person_unavailable::<RestState>),
        )
        .route(
            "/unavailable/{id}",
            delete(delete_sales_person_unavailable::<RestState>),
        )
        .route("/current", get(get_sales_person_current_user::<RestState>))
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    tags = ["Sales persons"],
    path = "",
    responses(
        (status = 200, description = "Get all sales persons", body = [SalesPersonTO]),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_all_sales_persons<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let sales_persons: Arc<[SalesPersonTO]> = rest_state
                .sales_person_service()
                .get_all(context.into(), None)
                .await?
                .iter()
                .map(SalesPersonTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&sales_persons).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}",
    tags = ["Sales persons"],
    description = "Get sales person by ID",
    params(
        ("id", description = "Sales person ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    responses(
        (status = 200, description = "Get sales person by ID", body = SalesPersonTO),
        (status = 404, description = "Sales person not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let sales_person = SalesPersonTO::from(
                &rest_state
                    .sales_person_service()
                    .get(sales_person_id, context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&sales_person).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["Sales persons"],
    description = "Create a new sales person",
    request_body = SalesPersonTO,
    responses(
        (status = 200, description = "Create sales person", body = SalesPersonTO),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn create_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(sales_person): Json<SalesPersonTO>,
) -> Response {
    error_handler(
        (async {
            let sales_person = SalesPersonTO::from(
                &rest_state
                    .sales_person_service()
                    .create(&(&sales_person).into(), context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&sales_person).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/{id}",
    tags = ["Sales persons"],
    description = "Update a sales person",
    params(
        ("id", description = "Sales person ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    request_body = SalesPersonTO,
    responses(
        (status = 200, description = "Update sales person", body = SalesPersonTO),
        (status = 404, description = "Sales person not found"),
        (status = 400, description = "Inconsistent ID"),
        (status = 422, description = "Validation error"),
        (status = 409, description = "Conflict"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn update_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
    Json(sales_person): Json<SalesPersonTO>,
) -> Response {
    error_handler(
        (async {
            if sales_person_id != sales_person.id {
                return Err(RestError::InconsistentId(sales_person_id, sales_person.id));
            }
            rest_state
                .sales_person_service()
                .update(&(&sales_person).into(), context.into(), None)
                .await?;
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&sales_person).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/{id}",
    tags = ["Sales persons"],
    description = "Delete a sales person",
    params(
        ("id", description = "Sales person ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    responses(
        (status = 204, description = "Delete sales person"),
        (status = 404, description = "Sales person not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn delete_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .sales_person_service()
                .delete(sales_person_id, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}/user",
    tags = ["Sales persons"],
    description = "Get sales person's username",
    params(
        ("id", description = "Sales person ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    responses(
        (status = 200, description = "Get sales person user", body = String),
        (status = 404, description = "Sales person not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_sales_person_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let user = rest_state
                .sales_person_service()
                .get_assigned_user(sales_person_id, context.into(), None)
                .await?;
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&user).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/{id}/user",
    tags = ["Sales persons"],
    description = "Set the username for a sales person",
    request_body (
        content = String,
        content_type = "application/json"
    ),
    params(
        ("id", description = "Sales person ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    responses(
        (status = 204, description = "Set sales person user"),
        (status = 404, description = "Sales person not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn set_sales_person_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
    Json(user): Json<Arc<str>>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .sales_person_service()
                .set_user(sales_person_id, user.into(), context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/{id}/user",
    tags = ["Sales persons"],
    description = "Delete the username for a sales person",
    params(
        ("id", description = "Sales person ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    responses(
        (status = 204, description = "Delete sales person user"),
        (status = 404, description = "Sales person not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn delete_sales_person_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .sales_person_service()
                .set_user(sales_person_id, None, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/current",
    tags = ["Sales persons"],
    description = "Get the sales persons for the current user",
    responses(
        (status = 200, description = "Get current user sales person", body = SalesPersonTO),
        (status = 404, description = "Sales person not found"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_sales_person_current_user<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let sales_person = rest_state
                .sales_person_service()
                .get_sales_person_current_user(context.into(), None)
                .await?
                .map(|sales_person| SalesPersonTO::from(&sales_person));

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&sales_person).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReportRequest {
    year: Option<u32>,
    calendar_week: Option<u8>,
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}/unavailable",
    tags = ["Sales persons"],
    description = "Get sales person unavailable information",
    params(
        ("id", description = "Sales person ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    responses(
        (status = 200, description = "Get sales person unavailable", body = [SalesPersonUnavailableTO]),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_sales_person_unavailable<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
    query: Query<ReportRequest>,
) -> Response {
    error_handler(
        (async {
            let tos = if let (Some(year), Some(calendar_week)) = (query.year, query.calendar_week) {
                rest_state
                    .sales_person_unavailable_service()
                    .get_by_week_for_sales_person(
                        sales_person_id,
                        year,
                        calendar_week,
                        context.into(),
                        None,
                    )
                    .await?
                    .iter()
                    .map(SalesPersonUnavailableTO::from)
                    .collect::<Vec<_>>()
            } else {
                rest_state
                    .sales_person_unavailable_service()
                    .get_all_for_sales_person(sales_person_id, context.into(), None)
                    .await?
                    .iter()
                    .map(SalesPersonUnavailableTO::from)
                    .collect::<Vec<_>>()
            };
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
    post,
    path = "/unavailable",
    tags = ["Sales persons"],
    description = "Set a new sales person unavailable information",
    request_body = SalesPersonUnavailableTO,
    params(
        ("id", description = "Sales person ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    responses(
        (status = 200, description = "Create sales person unavailable", body = SalesPersonUnavailableTO),
        (status = 404, description = "Sales person not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn create_sales_person_unavailable<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(sales_person_unavailable): Json<SalesPersonUnavailableTO>,
) -> Response {
    error_handler(
        (async {
            let unavailable = SalesPersonUnavailableTO::from(
                &rest_state
                    .sales_person_unavailable_service()
                    .create(&(&sales_person_unavailable).into(), context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&unavailable).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/unavailable/{id}",
    tags = ["Sales persons"],
    description = "Delete sales person unavailable information",
    params(
        ("id", description = "Sales person unavailable ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    responses(
        (status = 204, description = "Delete sales person unavailable"),
        (status = 404, description = "Sales person unavailable not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn delete_sales_person_unavailable<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(unavailable_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .sales_person_unavailable_service()
                .delete(unavailable_id, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}/ical",
    tags = ["Sales persons"],
    description = "Get sales person shift iCal export",
    params(
        ("id", description = "Sales person ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    responses(
        (status = 200, description = "Get sales person iCal", body = String),
        (status = 404, description = "Sales person not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn ical_for_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let ical = rest_state
                .block_service()
                .get_blocks_for_next_weeks_as_ical(sales_person_id, context.into(), None)
                .await?;
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "text/calendar")
                .body(Body::new(ical.as_ref().to_string()))
                .unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Sales persons", description = "Sales person API"),
    ),
    paths(
        get_all_sales_persons,
        get_sales_person,
        create_sales_person,
        update_sales_person,
        delete_sales_person,
        get_sales_person_user,
        set_sales_person_user,
        delete_sales_person_user,
        get_sales_person_unavailable,
        create_sales_person_unavailable,
        delete_sales_person_unavailable,
        get_sales_person_current_user,
        ical_for_sales_person,
    ),
    components(
        schemas(
            SalesPersonTO,
            SalesPersonUnavailableTO,
        ),
    ),
)]
pub struct SalesPersonApiDoc;
