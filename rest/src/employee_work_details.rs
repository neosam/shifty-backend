use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use rest_types::EmployeeWorkDetailsTO;

use service::employee_work_details::EmployeeWorkDetailsService;
use tracing::instrument;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route(
            "/for-week/{sales_person_id}/{year}/{calendar_week}",
            get(get_working_hours_for_week::<RestState>),
        )
        .route(
            "/for-sales-person/{sales_person_id}",
            get(get_working_hours_for_sales_person::<RestState>),
        )
        .route("/", post(create_working_hours::<RestState>))
        .route("/{id}", delete(delete_employee_work_details::<RestState>))
        .route("/{id}", put(update_working_hours::<RestState>))
}

#[instrument(skip(rest_state))]
pub async fn create_working_hours<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(working_hours): Json<EmployeeWorkDetailsTO>,
) -> Response {
    error_handler(
        (async {
            let working_hours = EmployeeWorkDetailsTO::from(
                &rest_state
                    .working_hours_service()
                    .create(&(&working_hours).into(), context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&working_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn update_working_hours<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(working_hours): Json<EmployeeWorkDetailsTO>,
) -> Response {
    error_handler(
        (async {
            let working_hours = EmployeeWorkDetailsTO::from(
                &rest_state
                    .working_hours_service()
                    .update(&(&working_hours).into(), context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&working_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn delete_employee_work_details<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .working_hours_service()
                .delete(id, context.into(), None)
                .await?;
            Ok(Response::builder().status(200).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn get_working_hours_for_week<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((sales_person_id, year, calendar_week)): Path<(Uuid, u32, u8)>,
) -> Response {
    error_handler(
        (async {
            let working_hours = EmployeeWorkDetailsTO::from(
                &rest_state
                    .working_hours_service()
                    .find_for_week(sales_person_id, calendar_week, year, context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&working_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn get_working_hours_for_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let working_hours: Arc<[EmployeeWorkDetailsTO]> = rest_state
                .working_hours_service()
                .find_by_sales_person_id(sales_person_id, context.into(), None)
                .await?
                .iter()
                .map(EmployeeWorkDetailsTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&working_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}
