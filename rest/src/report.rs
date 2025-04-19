use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    response::Response,
    routing::get,
    Extension, Router,
};
use rest_types::{EmployeeReportTO, ShortEmployeeReportTO};
use serde::Deserialize;
use service::reporting::ReportingService;
use tracing::instrument;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_short_report_for_all::<RestState>))
        .route(
            "/week/{year}/{calendar_week}",
            get(get_short_week_report::<RestState>),
        )
        .route("/{id}", get(get_report::<RestState>))
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReportRequest {
    year: u32,
    until_week: u8,
}

#[instrument(skip(rest_state))]
pub async fn get_short_report_for_all<RestState: RestStateDef>(
    rest_state: State<RestState>,
    query: Query<ReportRequest>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let short_report: Arc<[ShortEmployeeReportTO]> = rest_state
                .reporting_service()
                .get_reports_for_all_employees(query.year, query.until_week, context.into(), None)
                .await?
                .iter()
                .map(ShortEmployeeReportTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&short_report).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn get_report<RestState: RestStateDef>(
    rest_state: State<RestState>,
    query: Query<ReportRequest>,
    Path(sales_person_id): Path<Uuid>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let report: EmployeeReportTO = (&rest_state
                .reporting_service()
                .get_report_for_employee(
                    &sales_person_id,
                    query.year,
                    query.until_week,
                    context.into(),
                    None,
                )
                .await?)
                .into();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&report).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
pub async fn get_short_week_report<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((year, calendar_week)): Path<(u32, u8)>,
) -> Response {
    error_handler(
        (async {
            let report: Arc<[ShortEmployeeReportTO]> = rest_state
                .reporting_service()
                .get_week(year, calendar_week, context.into(), None)
                .await?
                .iter()
                .map(ShortEmployeeReportTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&report).unwrap()))
                .unwrap())
        })
        .await,
    )
}
