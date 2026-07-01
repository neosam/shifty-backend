use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{get, put},
    Extension, Json, Router,
};
use rest_types::{WeekStatusKindTO, WeekStatusTO};
use service::week_status::WeekStatusService;
use tracing::instrument;
use utoipa::OpenApi;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    // GET and PUT on the SAME path (D-39-06): the FE always sends year+week, there
    // is no id-based endpoint. Read is open to all roles, the write gate lives in
    // the service (set_week_status → SHIFTPLANNER_PRIVILEGE, T-39-01).
    Router::new()
        .route(
            "/by-year-and-week/{year}/{week}",
            get(get_week_status_by_year_and_week::<RestState>),
        )
        .route(
            "/by-year-and-week/{year}/{week}",
            put(upsert_week_status::<RestState>),
        )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/by-year-and-week/{year}/{week}",
    tags = ["Week Status"],
    params(
        ("year", description = "Year", example = "2025"),
        ("week", description = "Calendar week", example = "20"),
    ),
    responses(
        (status = 200, description = "Current week status (status=unset when no row)", body = WeekStatusTO),
    ),
)]
pub async fn get_week_status_by_year_and_week<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((year, week)): Path<(u32, u8)>,
) -> Response {
    error_handler(
        (async {
            let status = rest_state
                .week_status_service()
                .get_week_status(year, week, context.into(), None)
                .await?;
            let week_status_to = WeekStatusTO {
                year,
                calendar_week: week,
                status: WeekStatusKindTO::from(&status),
            };
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&week_status_to).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/by-year-and-week/{year}/{week}",
    tags = ["Week Status"],
    params(
        ("year", description = "Year", example = "2025"),
        ("week", description = "Calendar week", example = "20"),
    ),
    request_body = WeekStatusTO,
    responses(
        (status = 200, description = "Week status set (upsert)", body = WeekStatusTO),
        (status = 403, description = "Forbidden (not a shiftplanner)"),
    ),
)]
pub async fn upsert_week_status<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((year, week)): Path<(u32, u8)>,
    Json(body): Json<WeekStatusTO>,
) -> Response {
    error_handler(
        (async {
            // The permission gate is NOT duplicated here — it lives in the service
            // (set_week_status), which maps Forbidden → HTTP 403 via error_handler.
            let status = rest_state
                .week_status_service()
                .set_week_status(year, week, (&body.status).into(), context.into(), None)
                .await?;
            let week_status_to = WeekStatusTO {
                year,
                calendar_week: week,
                status: WeekStatusKindTO::from(&status),
            };
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&week_status_to).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(get_week_status_by_year_and_week, upsert_week_status,),
    components(schemas(WeekStatusTO, WeekStatusKindTO))
)]
pub struct WeekStatusApiDoc;
