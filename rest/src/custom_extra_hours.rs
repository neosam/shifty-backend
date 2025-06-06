use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    Extension, Json,
};
use rest_types::CustomExtraHoursTO;
use service::custom_extra_hours::CustomExtraHoursService;
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> axum::Router<RestState> {
    axum::Router::new()
        .route("/", axum::routing::get(get_all::<RestState>))
        .route("/{id}", axum::routing::get(get_by_id::<RestState>))
        .route(
            "/by-sales-person/{sales_person_id}",
            axum::routing::get(get_by_sales_person_id::<RestState>),
        )
        .route("/", axum::routing::post(create::<RestState>))
        .route("/{id}", axum::routing::put(update::<RestState>))
        .route("/{id}", axum::routing::delete(delete::<RestState>))
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "",
    tags = ["Custom Extra Hours"],
    responses(
        (status = 200, description = "Get all custom extra hours", body = [CustomExtraHoursTO]),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_all<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let weekly_summary: Arc<[CustomExtraHoursTO]> = rest_state
                .custom_extra_hours_service()
                .get_all(context.into(), None)
                .await?
                .iter()
                .map(CustomExtraHoursTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&weekly_summary).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}",
    tags = ["Custom Extra Hours"],
    responses(
        (status = 200, description = "Get custom extra hours by ID", body = CustomExtraHoursTO),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_by_id<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    id: Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let custom_extra_hours: CustomExtraHoursTO = rest_state
                .custom_extra_hours_service()
                .get_by_id(*id, context.into(), None)
                .await?
                .into();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(
                    serde_json::to_string(&custom_extra_hours).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/by-sales-person/{sales_person_id}",
    tags = ["Custom Extra Hours"],
    params(
        ("sales_person_id", description = "Sales person ID", example = "1a2b3c4d-5e6f-7g8h-9i0j-k1l2m3n4o5p6")
    ),
    responses(
        (status = 200, description = "Get custom extra hours for sales person", body = [CustomExtraHoursTO]),
        (status = 403, description = "Forbidden - insufficient permissions"),
        (status = 404, description = "Sales person not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_by_sales_person_id<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let custom_extra_hours: Arc<[CustomExtraHoursTO]> = rest_state
                .custom_extra_hours_service()
                .get_by_sales_person_id(sales_person_id, context.into(), None)
                .await?
                .iter()
                .map(CustomExtraHoursTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(
                    serde_json::to_string(&custom_extra_hours).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["Custom Extra Hours"],
    request_body = CustomExtraHoursTO,
    responses(
        (status = 201, description = "Create custom extra hours", body = CustomExtraHoursTO),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn create<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(custom_extra_hours): Json<CustomExtraHoursTO>,
) -> Response {
    error_handler(
        (async {
            let custom_extra_hours: CustomExtraHoursTO = rest_state
                .custom_extra_hours_service()
                .create(&custom_extra_hours.into(), context.into(), None)
                .await?
                .into();
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::new(
                    serde_json::to_string(&custom_extra_hours).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/{id}",
    tags = ["Custom Extra Hours"],
    request_body = CustomExtraHoursTO,
    responses(
        (status = 200, description = "Update custom extra hours", body = CustomExtraHoursTO),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn update<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    id: Path<Uuid>,
    Json(custom_extra_hours): Json<CustomExtraHoursTO>,
) -> Response {
    error_handler(
        (async {
            let custom_extra_hours: CustomExtraHoursTO = rest_state
                .custom_extra_hours_service()
                .update(&custom_extra_hours.into(), context.into(), None)
                .await?
                .into();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(
                    serde_json::to_string(&custom_extra_hours).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/custom-extra-hours/{id}",
    tags = ["Custom Extra Hours"],
    responses(
        (status = 204, description = "Delete custom extra hours"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn delete<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    id: Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .custom_extra_hours_service()
                .delete(*id, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Custom Extra Hours", description = "Custom Extra Hours API"),
    ),
    paths(
        get_all,
        get_by_id,
        get_by_sales_person_id,
        create,
        update,
        delete,
    ),
    components(
        schemas(
            CustomExtraHoursTO,
        ),
    ),
)]
pub struct CustomExtraHoursApiDoc;
