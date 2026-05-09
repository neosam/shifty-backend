//! Phase 4 — Cutover REST surface (Plan 04-06).
//!
//! Three POST endpoints under `/admin/cutover/*`:
//! - `gate-dry-run` (HR) — runs `CutoverService::run(dry_run=true)`; rolls back the cutover Tx.
//! - `commit`       (cutover_admin) — runs `CutoverService::run(dry_run=false)`; commits.
//! - `profile`      (HR — read-only) — runs `CutoverService::profile()`; full extra_hours scan
//!   that persists `.planning/migration-backup/profile-{ts}.json`.
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
    CutoverBulkConvertQuarantineRowsRequest, CutoverBulkConvertQuarantineRowsResponse,
    CutoverConvertErrorTO, CutoverConvertQuarantineEntryRequest,
    CutoverConvertQuarantineEntryResponse, CutoverGateDriftReportTO, CutoverGateDriftRowTO,
    CutoverProfileBucketTO, CutoverProfileTO, CutoverQuarantineEntryTO, CutoverRunResultTO,
    ExtraHoursCategoryDeprecatedErrorTO,
};
use serde::Deserialize;
use service::cutover::CutoverService;
use std::sync::Arc;
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
        // Phase 8.1 — atomic Convert endpoints (Plans 02 + 03 backend; this Plan 04 surfaces them).
        .route(
            "/convert-quarantine-entry",
            post(cutover_convert_quarantine_entry_handler::<RestState>),
        )
        .route(
            "/bulk-convert-quarantine-rows",
            post(cutover_bulk_convert_quarantine_rows_handler::<RestState>),
        )
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

/// Phase 8.1 (Plan 04, D-01) — Single-Convert endpoint.
///
/// Converts ONE quarantined `extra_hours` row into a fresh `absence_period`
/// row inside one atomic Tx. The `(from, to)` range is derived server-side
/// via the Plan 08-09 weekly-lump-sum heuristic — the frontend only sends
/// the `extra_hours_id`. Privilege: `cutover_admin`. On heuristic mismatch
/// the Tx rolls back and the handler returns 422.
///
/// The response carries an inline `refreshed_drift_report` (D-08, RESEARCH
/// P-03 option a) so the frontend can re-render the drift list without a
/// follow-up `gate-dry-run` roundtrip.
#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/convert-quarantine-entry",
    tags = ["Cutover"],
    request_body = CutoverConvertQuarantineEntryRequest,
    responses(
        (status = 200, description = "Single quarantined extra_hours row converted to absence_period in one atomic Tx.", body = CutoverConvertQuarantineEntryResponse),
        (status = 403, description = "Caller lacks cutover_admin privilege."),
        (status = 404, description = "extra_hours_id not found among not-yet-migrated rows (already migrated or deleted)."),
        (status = 422, description = "Row does not match the weekly-lump-sum heuristic; manual edit required."),
    ),
)]
pub async fn cutover_convert_quarantine_entry_handler<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    axum::Json(req): axum::Json<CutoverConvertQuarantineEntryRequest>,
) -> Response {
    error_handler(
        (async {
            let outcome = rest_state
                .cutover_service()
                .convert_quarantine_entry(req.extra_hours_id, context.into(), None)
                .await?;
            let body = CutoverConvertQuarantineEntryResponse::from(&outcome);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&body).unwrap()))
                .unwrap())
        })
        .await,
    )
}

/// Phase 8.1 (Plan 04, D-02) — strict-atomic Bulk-Convert endpoint.
///
/// Converts every quarantined `extra_hours` row matching the
/// `(sales_person_id, category, year)` triple — optionally narrowed by an
/// explicit `extra_hours_ids` subset — into `absence_period` rows in a
/// single Tx. All rows share one synthetic `cutover_run_id` for audit
/// cohesion. On any per-row heuristic mismatch the entire Tx rolls back
/// with 422. Empty match-set returns 404. Privilege: `cutover_admin`.
#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/bulk-convert-quarantine-rows",
    tags = ["Cutover"],
    request_body = CutoverBulkConvertQuarantineRowsRequest,
    responses(
        (status = 200, description = "All rows in the (sales_person, category, year) target set converted to absence_periods in one atomic Tx.", body = CutoverBulkConvertQuarantineRowsResponse),
        (status = 403, description = "Caller lacks cutover_admin privilege."),
        (status = 404, description = "No quarantined rows match the target triple."),
        (status = 422, description = "Strict-atomic — at least one row failed the heuristic; entire Tx rolled back."),
    ),
)]
pub async fn cutover_bulk_convert_quarantine_rows_handler<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    axum::Json(req): axum::Json<CutoverBulkConvertQuarantineRowsRequest>,
) -> Response {
    error_handler(
        (async {
            // Wire-tier `AbsenceCategoryTO` → service-tier `AbsenceCategory`.
            let category = service::absence::AbsenceCategory::from(&req.category);
            // `Option<Vec<Uuid>>` → `Option<Arc<[Uuid]>>` at the boundary
            // (D-02 trait param shape; cheap-clone semantics inside the hot path).
            let explicit_ids = req
                .extra_hours_ids
                .as_ref()
                .map(|ids| Arc::from(ids.clone().into_boxed_slice()));
            let outcome = rest_state
                .cutover_service()
                .bulk_convert_quarantine_rows(
                    req.sales_person_id,
                    category,
                    req.year,
                    explicit_ids,
                    context.into(),
                    None,
                )
                .await?;
            let body = CutoverBulkConvertQuarantineRowsResponse::from(&outcome);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&body).unwrap()))
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
        // Phase 8.1 — Convert + Bulk-Convert endpoints.
        cutover_convert_quarantine_entry_handler,
        cutover_bulk_convert_quarantine_rows_handler,
    ),
    components(schemas(
        CutoverRunResultTO,
        CutoverGateDriftRowTO,
        CutoverGateDriftReportTO,
        CutoverQuarantineEntryTO,
        CutoverProfileTO,
        CutoverProfileBucketTO,
        ExtraHoursCategoryDeprecatedErrorTO,
        CutoverGateDryRunRequest,
        CutoverCommitRequest,
        CutoverProfileRequest,
        // Phase 8.1 — Convert wire-DTOs.
        CutoverConvertQuarantineEntryRequest,
        CutoverConvertQuarantineEntryResponse,
        CutoverBulkConvertQuarantineRowsRequest,
        CutoverBulkConvertQuarantineRowsResponse,
        CutoverConvertErrorTO,
    )),
    tags(
        (name = "Cutover", description = "Phase-4 migration & cutover orchestration (admin only)."),
    ),
)]
pub struct CutoverApiDoc;
