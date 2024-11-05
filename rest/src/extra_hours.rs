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
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", post(create_extra_hours::<RestState>))
        .route("/:id", delete(delete_extra_hours::<RestState>))
        .route(
            "/by-sales-person/:id",
            get(get_extra_hours_for_sales_person::<RestState>),
        )
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExtraHoursForSalesPersonAttributes {
    year: u32,
    until_week: u8,
}

#[instrument(skip(rest_state))]
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
                )
                .await?
                .iter()
                .map(ExtraHoursTO::from)
                .collect();
            Ok(Response::builder()
                .status(201)
                .body(Body::new(serde_json::to_string(&extra_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
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
                    .create(&(&sales_person).into(), context.into())
                    .await?,
            );
            Ok(Response::builder()
                .status(201)
                .body(Body::new(serde_json::to_string(&extra_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn delete_extra_hours<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(extra_hours_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .extra_hours_service()
                .delete(extra_hours_id, context.into())
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}
