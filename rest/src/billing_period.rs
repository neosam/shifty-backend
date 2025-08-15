use std::sync::Arc;

use axum::body::Body;
use axum::extract::Path;
use axum::routing::{delete, get, post};
use axum::{extract::State, response::Response};
use axum::{Extension, Json, Router};
use rest_types::{
    BillingPeriodSalesPersonTO, BillingPeriodTO, BillingPeriodValueTO, CreateBillingPeriodRequestTO,
};
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};
use service::billing_period::BillingPeriodService;
use service::billing_period_report::BillingPeriodReportService;
use shifty_utils::ShiftyDate;

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_all_billing_periods::<RestState>))
        .route("/{id}", get(get_billing_period::<RestState>))
        .route("/", post(create_billing_period::<RestState>))
        .route("/", delete(clear_all_billing_periods::<RestState>))
}

#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "List all billing periods", body = Vec<BillingPeriodTO>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(rest_state))]
pub async fn get_all_billing_periods<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let billing_periods: Arc<[BillingPeriodTO]> = rest_state
                .billing_period_service()
                .get_billing_period_overview(context.into(), None)
                .await?
                .iter()
                .map(BillingPeriodTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&billing_periods).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[utoipa::path(
    get,
    path = "/{id}",
    params(
        ("id" = Uuid, Path, description = "Billing period ID")
    ),
    responses(
        (status = 200, description = "Get billing period by ID", body = BillingPeriodTO),
        (status = 404, description = "Billing period not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(rest_state))]
pub async fn get_billing_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let billing_period = rest_state
                .billing_period_service()
                .get_billing_period_by_id(id, context.into(), None)
                .await?;
            let billing_period_to = BillingPeriodTO::from(&billing_period);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(
                    serde_json::to_string(&billing_period_to).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[utoipa::path(
    post,
    path = "",
    request_body = CreateBillingPeriodRequestTO,
    responses(
        (status = 201, description = "Create new billing period", body = BillingPeriodTO),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - HR permission required"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(rest_state))]
pub async fn create_billing_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(payload): Json<CreateBillingPeriodRequestTO>,
) -> Response {
    error_handler(
        (async {
            let end_date = ShiftyDate::from_date(payload.end_date);
            let billing_period_id = rest_state
                .billing_period_report_service()
                .build_and_persist_billing_period_report(end_date, context.into(), None)
                .await?;
            Ok(Response::builder()
                .status(201)
                .body(Body::new(
                    serde_json::to_string(&billing_period_id).unwrap(),
                ))
                .unwrap())
        })
        .await,
    )
}

#[utoipa::path(
    delete,
    path = "",
    responses(
        (status = 204, description = "All billing periods cleared successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - HR permission required"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(rest_state))]
pub async fn clear_all_billing_periods<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .billing_period_service()
                .clear_all_billing_periods(context.into(), None)
                .await?;
            Ok(Response::builder()
                .status(204)
                .body(Body::empty())
                .unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_billing_periods,
        get_billing_period,
        create_billing_period,
        clear_all_billing_periods,
    ),
    components(
        schemas(BillingPeriodTO, BillingPeriodSalesPersonTO, BillingPeriodValueTO, CreateBillingPeriodRequestTO)
    ),
    tags(
        (name = "billing_period", description = "Billing Period management")
    )
)]
pub struct BillingPeriodApiDoc;
