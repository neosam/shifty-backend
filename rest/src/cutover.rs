//! Phase 4 — Cutover REST surface (Plan 04-06).
//!
//! Three POST endpoints under `/admin/cutover/*`:
//! - `gate-dry-run` (HR) — runs `CutoverService::run(dry_run=true)`; rolls back the cutover Tx.
//! - `commit`       (cutover_admin) — runs `CutoverService::run(dry_run=false)`; commits.
//! - `profile`      (HR — read-only) — runs `CutoverService::profile()`; full extra_hours scan
//!                   that persists `.planning/migration-backup/profile-{ts}.json`.
//!
//! Permission enforcement happens at the service layer (D-Phase4-07 + Pattern 3 in
//! 04-RESEARCH.md). REST handlers are thin shims: serialize the result via the
//! Phase-3 `From<&service::cutover::*>` conversions in `rest-types`.

use axum::{
    body::Body,
    extract::State,
    response::Response,
    routing::post,
    Extension, Router,
};
use rest_types::{
    CutoverProfileBucketTO, CutoverProfileTO, CutoverRunResultTO,
    ExtraHoursCategoryDeprecatedErrorTO,
};
use serde::Deserialize;
use service::cutover::CutoverService;
use tracing::instrument;
use utoipa::OpenApi;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route(
            "/gate-dry-run",
            post(cutover_gate_dry_run_handler::<RestState>),
        )
        .route("/commit", post(cutover_commit_handler::<RestState>))
        .route("/profile", post(cutover_profile_handler::<RestState>))
}

/// Empty placeholder — kept as a typed body so future field additions remain
/// backward-compatible. Adding a non-Option field here IS a snapshot-breaking
/// change (the OpenAPI schema diff will fire) and requires explicit review.
#[derive(Clone, Debug, Deserialize, utoipa::ToSchema)]
pub struct CutoverGateDryRunRequest {}

/// Empty placeholder — see `CutoverGateDryRunRequest` for snapshot-impact note.
#[derive(Clone, Debug, Deserialize, utoipa::ToSchema)]
pub struct CutoverCommitRequest {}

/// Empty placeholder — see `CutoverGateDryRunRequest` for snapshot-impact note.
#[derive(Clone, Debug, Deserialize, utoipa::ToSchema)]
pub struct CutoverProfileRequest {}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/gate-dry-run",
    tags = ["Cutover"],
    responses(
        (status = 200, description = "Dry-run cutover result (Tx rolled back).", body = CutoverRunResultTO),
        (status = 403, description = "Caller lacks HR privilege."),
    ),
)]
pub async fn cutover_gate_dry_run_handler<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let result = rest_state
                .cutover_service()
                .run(true, context.into(), None)
                .await?;
            let to = CutoverRunResultTO::from(&result);
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
    post,
    path = "/commit",
    tags = ["Cutover"],
    responses(
        (status = 200, description = "Cutover committed (atomic Tx: migration + carryover refresh + soft-delete + flag-flip).", body = CutoverRunResultTO),
        (status = 403, description = "Caller lacks cutover_admin privilege."),
    ),
)]
pub async fn cutover_commit_handler<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let result = rest_state
                .cutover_service()
                .run(false, context.into(), None)
                .await?;
            let to = CutoverRunResultTO::from(&result);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
                .unwrap())
        })
        .await,
    )
}

/// SC-1 surface — Production-Data-Profile. Read-only from a state perspective
/// (the cutover Tx is rolled back inside `profile()`); persists a JSON file
/// under `.planning/migration-backup/profile-{ts}.json` for HR review.
/// Permission: HR (matches gate-dry-run; profile is non-destructive).
#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/profile",
    tags = ["Cutover"],
    responses(
        (status = 200, description = "Production-data profile generated; JSON file persisted under .planning/migration-backup/.", body = CutoverProfileTO),
        (status = 403, description = "Caller lacks HR privilege."),
    ),
)]
pub async fn cutover_profile_handler<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let profile = rest_state
                .cutover_service()
                .profile(context.into(), None)
                .await?;
            let to = CutoverProfileTO::from(&profile);
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
    paths(
        cutover_gate_dry_run_handler,
        cutover_commit_handler,
        cutover_profile_handler,
    ),
    components(schemas(
        CutoverRunResultTO,
        CutoverProfileTO,
        CutoverProfileBucketTO,
        ExtraHoursCategoryDeprecatedErrorTO,
        CutoverGateDryRunRequest,
        CutoverCommitRequest,
        CutoverProfileRequest,
    )),
    tags(
        (name = "Cutover", description = "Phase-4 migration & cutover orchestration (admin only)."),
    ),
)]
pub struct CutoverApiDoc;
