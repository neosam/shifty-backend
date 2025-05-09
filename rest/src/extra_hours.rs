use std::rc::Rc;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    response::Response,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use rest_types::ExtraHoursTO;

use serde::Deserialize;
use service::extra_hours::ExtraHoursService;
use tracing::instrument;
use utoipa::{IntoParams, OpenApi};
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", post(create_extra_hours::<RestState>))
        .route("/{id}", delete(delete_extra_hours::<RestState>))
        .route(
            "/by-sales-person/{id}",
            get(get_extra_hours_for_sales_person::<RestState>),
        )
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ExtraHoursForSalesPersonAttributes {
    #[param(example = "2025")]
    year: u32,

    #[param(example = "20")]
    until_week: u8,
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/by-sales-person/{id}",

    tags = ["Extra Hours"],
    params(
        ("id", description = "Sales person id", example = "1a2b3c4d-5e6f-7g8h-9i0j-k1l2m3n4o5p6"),
        ExtraHoursForSalesPersonAttributes,
    ),
    responses(
        (status = 201, description = "Extra hours for sales person", body = [ExtraHoursTO]),
        (status = 404, description = "Sales person not found"),
    ),
)]
pub async fn get_extra_hours_for_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    query: Query<ExtraHoursForSalesPersonAttributes>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let extra_hours: Rc<[ExtraHoursTO]> = rest_state
                .extra_hours_service()
                .find_by_sales_person_id_and_year(
                    sales_person_id,
                    query.year,
                    query.until_week,
                    context.into(),
                    None,
                )
                .await?
                .iter()
                .map(ExtraHoursTO::from)
                .collect();
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&extra_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["Extra Hours"],
    request_body = ExtraHoursTO,
    responses(
        (status = 201, description = "Extra hours created", body = ExtraHoursTO),
        (status = 400, description = "Invalid input"),
    ),
)]
pub async fn create_extra_hours<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(sales_person): Json<ExtraHoursTO>,
) -> Response {
    error_handler(
        (async {
            let extra_hours = ExtraHoursTO::from(
                &rest_state
                    .extra_hours_service()
                    .create(&(&sales_person).into(), context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&extra_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/{id}",
    tags = ["Extra Hours"],
    params(
        ("id", description = "Extra hours id", example = "1a2b3c4d-5e6f-7g8h-9i0j-k1l2m3n4o5p6"),
    ),
    responses(
        (status = 204, description = "Extra hours deleted"),
        (status = 404, description = "Extra hours not found"),
    ),
)]
pub async fn delete_extra_hours<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(extra_hours_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .extra_hours_service()
                .delete(extra_hours_id, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_extra_hours_for_sales_person,
        create_extra_hours,
        delete_extra_hours
    ),
    components(schemas(ExtraHoursTO)),
    tags(
        (name = "Extra Hours", description = "Extra hours management"),
    ),
)]
pub struct ExtraHoursApiDoc;
