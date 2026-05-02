use axum::{
    extract::{Path, State},
    routing::get,
    Extension, Router,
};
use rest_types::{
    AbsenceCategoryTO, DayOfWeekTO, ShiftplanDayAggregateTO, ShiftplanWeekTO,
    UnavailabilityMarkerTO,
};
use utoipa::OpenApi;

use crate::{error_handler, Context, Response, RestStateDef};
use service::{permission::Authentication, shiftplan::ShiftplanViewService};
use uuid::Uuid;

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/{shiftplan_id}/{year}/{week}", get(get_shiftplan_week::<RestState>))
        .route("/day/{year}/{week}/{day_of_week}", get(get_shiftplan_day::<RestState>))
        // Phase-3 (D-Phase3-12) — per-sales-person-Sicht. Setzt
        // `ShiftplanDayTO.unavailable` für jeden Tag, an dem für den
        // Mitarbeiter eine aktive AbsencePeriod und/oder ein
        // sales_person_unavailable-Eintrag existiert (Pitfall-1 / SC4
        // gefiltert: soft-deleted ignoriert).
        .route(
            "/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}",
            get(get_shiftplan_week_for_sales_person::<RestState>),
        )
        .route(
            "/day/{year}/{week}/{day_of_week}/sales-person/{sales_person_id}",
            get(get_shiftplan_day_for_sales_person::<RestState>),
        )
}

#[utoipa::path(
    get,
    path = "/{shiftplan_id}/{year}/{week}",
    params(
        ("shiftplan_id" = Uuid, Path, description = "Shift plan ID"),
        ("year" = u32, Path, description = "Year of the shift plan"),
        ("week" = u8, Path, description = "Calendar week number (1-53)")
    ),
    responses(
        (status = 200, description = "Shift plan for the specified week", body = ShiftplanWeekTO),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "shiftplan"
)]
async fn get_shiftplan_week<RestState: RestStateDef>(
    Path((shiftplan_id, year, week)): Path<(Uuid, u32, u8)>,
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        async {
            let shiftplan = rest_state
                .shiftplan_view_service()
                .get_shiftplan_week(shiftplan_id, year, week, Authentication::Context(context), None)
                .await?;

            let shiftplan_to = ShiftplanWeekTO::from(&shiftplan);

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&shiftplan_to).unwrap(),
                ))
                .unwrap())
        }
        .await,
    )
}

#[utoipa::path(
    get,
    path = "/day/{year}/{week}/{day_of_week}",
    params(
        ("year" = u32, Path, description = "Year of the shift plan"),
        ("week" = u8, Path, description = "Calendar week number (1-53)"),
        ("day_of_week" = DayOfWeekTO, Path, description = "Day of the week (Monday, Tuesday, etc.)")
    ),
    responses(
        (status = 200, description = "Aggregated shift plans for the specified day", body = ShiftplanDayAggregateTO),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "shiftplan"
)]
async fn get_shiftplan_day<RestState: RestStateDef>(
    Path((year, week, day_of_week)): Path<(u32, u8, DayOfWeekTO)>,
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        async {
            let aggregate = rest_state
                .shiftplan_view_service()
                .get_shiftplan_day(year, week, day_of_week.into(), Authentication::Context(context), None)
                .await?;

            let aggregate_to = ShiftplanDayAggregateTO::from(&aggregate);

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::to_string(&aggregate_to).unwrap(),
                ))
                .unwrap())
        }
        .await,
    )
}

/// Phase 3 — per-sales-person-Wochen-Sicht (PLAN-01).
///
/// Liefert die Schichtplan-Woche, wobei jeder Tag das Feld
/// `unavailable: Option<UnavailabilityMarkerTO>` gesetzt bekommt, falls
/// für `sales_person_id` an diesem Tag eine aktive AbsencePeriod
/// und/oder ein aktiver `sales_person_unavailable`-Eintrag existiert.
/// Soft-deleted Einträge werden gefiltert (Pitfall-1 / SC4).
///
/// Permission: HR ∨ `verify_user_is_sales_person(sales_person_id)`
/// (D-Phase3-12).
#[utoipa::path(
    get,
    path = "/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}",
    params(
        ("shiftplan_id" = Uuid, Path, description = "Shift plan ID"),
        ("year" = u32, Path, description = "Year of the shift plan"),
        ("week" = u8, Path, description = "Calendar week number (1-53)"),
        ("sales_person_id" = Uuid, Path, description = "Sales person id (HR ∨ self)"),
    ),
    responses(
        (status = 200, description = "Shift plan week with per-day unavailable marker for the given sales person", body = ShiftplanWeekTO),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    tag = "shiftplan"
)]
async fn get_shiftplan_week_for_sales_person<RestState: RestStateDef>(
    Path((shiftplan_id, year, week, sales_person_id)): Path<(Uuid, u32, u8, Uuid)>,
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        async {
            let shiftplan = rest_state
                .shiftplan_view_service()
                .get_shiftplan_week_for_sales_person(
                    shiftplan_id,
                    year,
                    week,
                    sales_person_id,
                    Authentication::Context(context),
                    None,
                )
                .await?;

            let to = ShiftplanWeekTO::from(&shiftplan);

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&to).unwrap()))
                .unwrap())
        }
        .await,
    )
}

/// Phase 3 — per-sales-person-Tages-Sicht (PLAN-01). Analog zu
/// [`get_shiftplan_week_for_sales_person`], aber liefert nur den einen
/// `day_of_week` als Aggregat über alle Shiftplans.
///
/// Permission: HR ∨ `verify_user_is_sales_person(sales_person_id)`
/// (D-Phase3-12).
#[utoipa::path(
    get,
    path = "/day/{year}/{week}/{day_of_week}/sales-person/{sales_person_id}",
    params(
        ("year" = u32, Path, description = "Year of the shift plan"),
        ("week" = u8, Path, description = "Calendar week number (1-53)"),
        ("day_of_week" = DayOfWeekTO, Path, description = "Day of the week (Monday, Tuesday, etc.)"),
        ("sales_person_id" = Uuid, Path, description = "Sales person id (HR ∨ self)"),
    ),
    responses(
        (status = 200, description = "Aggregated shift plans for the specified day with per-day unavailable marker", body = ShiftplanDayAggregateTO),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    tag = "shiftplan"
)]
async fn get_shiftplan_day_for_sales_person<RestState: RestStateDef>(
    Path((year, week, day_of_week, sales_person_id)): Path<(u32, u8, DayOfWeekTO, Uuid)>,
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        async {
            let aggregate = rest_state
                .shiftplan_view_service()
                .get_shiftplan_day_for_sales_person(
                    year,
                    week,
                    day_of_week.into(),
                    sales_person_id,
                    Authentication::Context(context),
                    None,
                )
                .await?;

            let to = ShiftplanDayAggregateTO::from(&aggregate);

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&to).unwrap()))
                .unwrap())
        }
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_shiftplan_week,
        get_shiftplan_day,
        get_shiftplan_week_for_sales_person,
        get_shiftplan_day_for_sales_person,
    ),
    components(
        schemas(
            ShiftplanWeekTO,
            ShiftplanDayAggregateTO,
            UnavailabilityMarkerTO,
            AbsenceCategoryTO,
        )
    ),
    tags(
        (name = "shiftplan", description = "Shift plan management")
    )
)]
pub struct ShiftplanApiDoc;
