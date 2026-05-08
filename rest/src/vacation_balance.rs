//! REST-Layer für die Vacation-Balance-Domain (Phase 8 — Resturlaubs-Endpoint).
//!
//! Zwei Routen unter `/vacation-balance`:
//! - `GET /{sales_person_id}/{year}` — HR ∨ self (T-8-AUTH-01, T-8-IDOR-01).
//! - `GET /team/{year}` — HR-only (T-8-AUTH-02).
//!
//! Jeder Handler trägt `#[utoipa::path]` (CC-06) +
//! `#[instrument(skip(rest_state))]`. Permission-Enforcement passiert im
//! Service-Layer (`service_impl::vacation_balance`); der REST-Layer ist ein
//! dünner Wrapper mit DTO-Conversion und Error-Mapping via `error_handler`.
//!
//! Routen-Reihenfolge wichtig: `/team/{year}` MUSS vor
//! `/{sales_person_id}/{year}` stehen, sonst routet Axum `team` als
//! `sales_person_id` (Uuid-Parse-Error → 400 statt der gewünschten
//! Team-Route).

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::get,
    Extension, Router,
};
use rest_types::VacationBalanceTO;
use service::vacation_balance::VacationBalanceService;
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route(
            "/team/{year}",
            get(get_team_vacation_balance::<RestState>),
        )
        .route(
            "/{sales_person_id}/{year}",
            get(get_vacation_balance::<RestState>),
        )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{sales_person_id}/{year}",
    tags = ["VacationBalance"],
    params(
        ("sales_person_id" = Uuid, Path, description = "Sales person ID"),
        ("year" = u32, Path, description = "Calendar year (e.g. 2026)"),
    ),
    responses(
        (status = 200, description = "Vacation balance for sales person + year", body = VacationBalanceTO),
        (status = 403, description = "Forbidden — not HR and not self"),
        (status = 404, description = "Sales person not found"),
    ),
)]
pub async fn get_vacation_balance<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((sales_person_id, year)): Path<(Uuid, u32)>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.vacation_balance_service();
            let balance = svc
                .get(sales_person_id, year, context.into(), None)
                .await?;
            let to = VacationBalanceTO::from(&balance);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/team/{year}",
    tags = ["VacationBalance"],
    params(("year" = u32, Path, description = "Calendar year")),
    responses(
        (status = 200, description = "Team vacation balance (HR aggregate)", body = [VacationBalanceTO]),
        (status = 403, description = "Forbidden — HR-only"),
    ),
)]
pub async fn get_team_vacation_balance<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(year): Path<u32>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.vacation_balance_service();
            let balances = svc.get_team(year, context.into(), None).await?;
            let tos: Vec<VacationBalanceTO> =
                balances.iter().map(VacationBalanceTO::from).collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&tos).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(get_vacation_balance, get_team_vacation_balance),
    components(schemas(VacationBalanceTO)),
    tags(
        (
            name = "VacationBalance",
            description = "Vacation balance aggregate (entitled / carryover / used / planned / remaining)",
        ),
    ),
)]
pub struct VacationBalanceApiDoc;
