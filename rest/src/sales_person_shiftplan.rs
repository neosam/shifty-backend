use axum::body::Body;
use axum::extract::Path;
use axum::routing::{get, put};
use axum::{extract::State, response::Response};
use axum::{Extension, Json, Router};
use rest_types::SalesPersonTO;
use service::sales_person_shiftplan::SalesPersonShiftplanService;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route(
            "/{id}/shiftplans",
            get(get_shiftplans_for_sales_person::<RestState>),
        )
        .route(
            "/{id}/shiftplans",
            put(set_shiftplans_for_sales_person::<RestState>),
        )
        .route(
            "/by-shiftplan/{shiftplan_id}",
            get(get_bookable_sales_persons::<RestState>),
        )
}

#[utoipa::path(
    get,
    path = "/{id}/shiftplans",
    tags = ["Sales person shiftplan assignment"],
    description = "Get assigned shiftplan IDs for a sales person",
    params(
        ("id", description = "Sales person ID"),
    ),
    responses(
        (status = 200, description = "List of assigned shiftplan IDs", body = Vec<Uuid>),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error"),
    ),
)]
async fn get_shiftplans_for_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let shiftplan_ids = rest_state
                .sales_person_shiftplan_service()
                .get_shiftplans_for_sales_person(id, context.into(), None)
                .await?;
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&shiftplan_ids).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[utoipa::path(
    put,
    path = "/{id}/shiftplans",
    tags = ["Sales person shiftplan assignment"],
    description = "Set shiftplan assignments for a sales person (replaces all)",
    params(
        ("id", description = "Sales person ID"),
    ),
    request_body(
        content = Vec<Uuid>,
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "Assignments updated"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Sales person not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
async fn set_shiftplans_for_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
    Json(shiftplan_ids): Json<Vec<Uuid>>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .sales_person_shiftplan_service()
                .set_shiftplans_for_sales_person(id, &shiftplan_ids, context.into(), None)
                .await?;
            Ok(Response::builder().status(200).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[utoipa::path(
    get,
    path = "/by-shiftplan/{shiftplan_id}",
    tags = ["Sales person shiftplan assignment"],
    description = "Get sales persons eligible to be booked in a shiftplan (permissive model)",
    params(
        ("shiftplan_id", description = "Shiftplan ID"),
    ),
    responses(
        (status = 200, description = "List of bookable sales persons", body = Vec<SalesPersonTO>),
        (status = 500, description = "Internal server error"),
    ),
)]
async fn get_bookable_sales_persons<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(shiftplan_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let sales_persons: Vec<SalesPersonTO> = rest_state
                .sales_person_shiftplan_service()
                .get_bookable_sales_persons(shiftplan_id, context.into(), None)
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

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Sales person shiftplan assignment", description = "Manage sales person to shiftplan assignments"),
    ),
    paths(
        get_shiftplans_for_sales_person,
        set_shiftplans_for_sales_person,
        get_bookable_sales_persons,
    ),
    components(
        schemas(
            SalesPersonTO,
        ),
    ),
)]
pub struct SalesPersonShiftplanApiDoc;
