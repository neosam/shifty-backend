use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, Response, RestStateDef};
use rest_types::ShiftplanTO;
use service::{permission::Authentication, shiftplan_catalog::ShiftplanService};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_all_shiftplans::<RestState>))
        .route("/", post(create_shiftplan::<RestState>))
        .route("/{id}", get(get_shiftplan::<RestState>))
        .route("/{id}", put(update_shiftplan::<RestState>))
        .route("/{id}", delete(delete_shiftplan::<RestState>))
}

#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "List all shift plans", body = [ShiftplanTO]),
        (status = 500, description = "Internal server error")
    ),
    tag = "shiftplan-catalog"
)]
async fn get_all_shiftplans<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let shiftplans: Vec<ShiftplanTO> = rest_state
                .shiftplan_service()
                .get_all(Authentication::Context(context), None)
                .await?
                .iter()
                .map(ShiftplanTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::new(
                    serde_json::to_string(&shiftplans).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[utoipa::path(
    get,
    path = "/{id}",
    params(
        ("id" = Uuid, Path, description = "Shift plan ID")
    ),
    responses(
        (status = 200, description = "Get shift plan by ID", body = ShiftplanTO),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "shiftplan-catalog"
)]
async fn get_shiftplan<RestState: RestStateDef>(
    Path(id): Path<Uuid>,
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let shiftplan = rest_state
                .shiftplan_service()
                .get_by_id(id, Authentication::Context(context), None)
                .await?;
            let to = ShiftplanTO::from(&shiftplan);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::new(
                    serde_json::to_string(&to).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[utoipa::path(
    post,
    path = "",
    request_body = ShiftplanTO,
    responses(
        (status = 200, description = "Shift plan created", body = ShiftplanTO),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    tag = "shiftplan-catalog"
)]
async fn create_shiftplan<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(body): Json<ShiftplanTO>,
) -> Response {
    error_handler(
        (async {
            let shiftplan = service::shiftplan_catalog::Shiftplan::from(&body);
            let created = rest_state
                .shiftplan_service()
                .create(&shiftplan, Authentication::Context(context), None)
                .await?;
            let to = ShiftplanTO::from(&created);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::new(
                    serde_json::to_string(&to).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[utoipa::path(
    put,
    path = "/{id}",
    params(
        ("id" = Uuid, Path, description = "Shift plan ID")
    ),
    request_body = ShiftplanTO,
    responses(
        (status = 200, description = "Shift plan updated", body = ShiftplanTO),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Conflict"),
        (status = 500, description = "Internal server error")
    ),
    tag = "shiftplan-catalog"
)]
async fn update_shiftplan<RestState: RestStateDef>(
    Path(id): Path<Uuid>,
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(body): Json<ShiftplanTO>,
) -> Response {
    error_handler(
        (async {
            let mut shiftplan = service::shiftplan_catalog::Shiftplan::from(&body);
            shiftplan.id = id;
            let updated = rest_state
                .shiftplan_service()
                .update(&shiftplan, Authentication::Context(context), None)
                .await?;
            let to = ShiftplanTO::from(&updated);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::new(
                    serde_json::to_string(&to).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[utoipa::path(
    delete,
    path = "/{id}",
    params(
        ("id" = Uuid, Path, description = "Shift plan ID")
    ),
    responses(
        (status = 200, description = "Shift plan deleted"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "shiftplan-catalog"
)]
async fn delete_shiftplan<RestState: RestStateDef>(
    Path(id): Path<Uuid>,
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .shiftplan_service()
                .delete(id, Authentication::Context(context), None)
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(axum::body::Body::empty())
                .unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_shiftplans,
        get_shiftplan,
        create_shiftplan,
        update_shiftplan,
        delete_shiftplan,
    ),
    components(
        schemas(
            ShiftplanTO,
        )
    ),
    tags(
        (name = "shiftplan-catalog", description = "Shift plan catalog management")
    )
)]
pub struct ShiftplanCatalogApiDoc;
