use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    response::Response,
    routing::get,
    Extension, Router,
};
use rest_types::{
    EmployeeAttendanceStatisticsTO, EmployeeReportTO, EmployeeWeeklyStatisticsTO,
    ShortEmployeeReportTO, VoluntaryStatsTO, WeekdayAttendanceTO,
};
use serde::Deserialize;
use service::reporting::ReportingService;
use service::voluntary_stats::VoluntaryStatsService;
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_short_report_for_all::<RestState>))
        .route(
            "/week/{year}/{calendar_week}",
            get(get_short_week_report::<RestState>),
        )
        .route("/{id}/weekly-statistics", get(get_weekly_statistics::<RestState>))
        .route(
            "/{id}/attendance-statistics",
            get(get_attendance_statistics::<RestState>),
        )
        .route(
            "/{id}/voluntary-stats",
            get(get_voluntary_stats::<RestState>),
        )
        .route("/{id}", get(get_report::<RestState>))
}

#[derive(Clone, Debug, Deserialize, utoipa::ToSchema)]
pub struct ReportRequest {
    year: u32,
    until_week: u8,
}

/// Phase 54 Gap-Closure G1 (VOL-STAT-01 / VOL-ACCT-01/02) — Query-Parameter
/// fuer `GET /report/{id}/voluntary-stats`. Analog
/// `ReportingService::get_report_for_employee_range` eine echte Date-Range:
/// `from_date` und `to_date` als ISO YYYY-MM-DD.
///
/// Die Aggregation laeuft ueber alle Tage in `[from_date ..= to_date]`;
/// Edge-Weeks tragen tages-genau bei (Pro-Rata bleibt Tages-basiert, D-F2-01).
#[derive(Clone, Debug, Deserialize, utoipa::ToSchema)]
pub struct VoluntaryStatsRequest {
    /// ISO date YYYY-MM-DD (inclusive lower bound).
    pub from_date: String,
    /// ISO date YYYY-MM-DD (inclusive upper bound).
    pub to_date: String,
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "",
    tags = ["Report"],
    params(
        ("year" = u32, Query, description = "The year for the report"),
        ("until_week" = u8, Query, description = "The week to report until")
    ),
    responses(
        (status = 200, description = "Get short report for all employees", body = [ShortEmployeeReportTO], content_type = "application/json"),
        (status = 500, description = "Internal server error")
    )
)]
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
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&short_report).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}",
    tags = ["Report"],
    params(
        ("id" = Uuid, Path, description = "Sales person ID"),
        ("year" = u32, Query, description = "The year for the report"),
        ("until_week" = u8, Query, description = "The week to report until")
    ),
    responses(
        (status = 200, description = "Get report for an employee", body = EmployeeReportTO, content_type = "application/json"),
        (status = 500, description = "Internal server error")
    )
)]
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
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&report).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/week/{year}/{calendar_week}",
    tags = ["Report"],
    params(
        ("year" = u32, Path, description = "The year for the report"),
        ("calendar_week" = u8, Path, description = "The calendar week for the report")
    ),
    responses(
        (status = 200, description = "Get short week report", body = [ShortEmployeeReportTO]),
        (status = 500, description = "Internal server error")
    )
)]
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

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}/weekly-statistics",
    tags = ["Report"],
    params(
        ("id" = Uuid, Path, description = "Sales person ID")
    ),
    responses(
        (status = 200, description = "HR-only average worked hours per week", body = EmployeeWeeklyStatisticsTO, content_type = "application/json"),
        (status = 403, description = "Forbidden — HR role required"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_weekly_statistics<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Path(sales_person_id): Path<Uuid>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let stats: EmployeeWeeklyStatisticsTO = (&rest_state
                .reporting_service()
                .get_employee_weekly_statistics(&sales_person_id, context.into(), None)
                .await?)
                .into();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&stats).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}/attendance-statistics",
    tags = ["Report"],
    params(
        ("id" = Uuid, Path, description = "Sales person ID"),
        ("year" = u32, Query, description = "The year for the report"),
        ("until_week" = u8, Query, description = "The week to report until")
    ),
    responses(
        (status = 200, description = "HR-only per-weekday attendance-day distribution (count + share) over the report range for flexible employees; null body for non-flexible employees", body = inline(Option<EmployeeAttendanceStatisticsTO>), content_type = "application/json"),
        (status = 403, description = "Forbidden — HR role required"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_attendance_statistics<RestState: RestStateDef>(
    rest_state: State<RestState>,
    query: Query<ReportRequest>,
    Path(sales_person_id): Path<Uuid>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let maybe_stats = rest_state
                .reporting_service()
                .get_employee_attendance_statistics(
                    &sales_person_id,
                    query.year,
                    query.until_week,
                    context.into(),
                    None,
                )
                .await?;
            let stats: Option<EmployeeAttendanceStatisticsTO> =
                maybe_stats.as_ref().map(EmployeeAttendanceStatisticsTO::from);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&stats).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}/voluntary-stats",
    tags = ["Report"],
    params(
        ("id" = Uuid, Path, description = "Sales person ID"),
        ("from_date" = String, Query, description = "ISO date YYYY-MM-DD, inclusive lower bound"),
        ("to_date" = String, Query, description = "ISO date YYYY-MM-DD, inclusive upper bound")
    ),
    responses(
        (status = 200, description = "HR-only voluntary hours statistics for the given date range; Non-HR receives all fields as null (API-level redaction)", body = VoluntaryStatsTO, content_type = "application/json"),
        (status = 400, description = "Invalid date format (expected YYYY-MM-DD) or from_date > to_date"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_voluntary_stats<RestState: RestStateDef>(
    rest_state: State<RestState>,
    query: Query<VoluntaryStatsRequest>,
    Path(sales_person_id): Path<Uuid>,
    Extension(context): Extension<Context>,
) -> Response {
    // Parse ISO YYYY-MM-DD (Praezedenz rest/src/toggle.rs Zeilen 350-377).
    let parse_iso = |s: &str| -> Option<time::Date> {
        if s.len() != 10 {
            return None;
        }
        let b = s.as_bytes();
        if b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let year: i32 = s[0..4].parse().ok()?;
        let month: u8 = s[5..7].parse().ok()?;
        let day: u8 = s[8..10].parse().ok()?;
        time::Date::from_calendar_date(year, time::Month::try_from(month).ok()?, day).ok()
    };
    let (Some(from_naive), Some(to_naive)) =
        (parse_iso(&query.from_date), parse_iso(&query.to_date))
    else {
        return Response::builder()
            .status(400)
            .header("Content-Type", "text/plain")
            .body(Body::new(
                "Invalid ISO date format. Expected YYYY-MM-DD.".to_string(),
            ))
            .unwrap();
    };
    if from_naive > to_naive {
        return Response::builder()
            .status(400)
            .header("Content-Type", "text/plain")
            .body(Body::new("from_date must be <= to_date.".to_string()))
            .unwrap();
    }
    let from_date = shifty_utils::ShiftyDate::from_date(from_naive);
    let to_date = shifty_utils::ShiftyDate::from_date(to_naive);
    error_handler(
        (async {
            let stats = rest_state
                .voluntary_stats_service()
                .get_voluntary_stats(
                    sales_person_id,
                    from_date,
                    to_date,
                    context.into(),
                    None,
                )
                .await?;
            let to = VoluntaryStatsTO::from(&stats);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Report", description = "Report API")
    ),
    paths(
        get_short_report_for_all,
        get_report,
        get_short_week_report,
        get_weekly_statistics,
        get_attendance_statistics,
        get_voluntary_stats
    ),
    components(schemas(
        ShortEmployeeReportTO,
        EmployeeReportTO,
        ReportRequest,
        EmployeeWeeklyStatisticsTO,
        EmployeeAttendanceStatisticsTO,
        WeekdayAttendanceTO,
        VoluntaryStatsTO,
        VoluntaryStatsRequest
    ))
)]
pub struct ReportApiDoc;
