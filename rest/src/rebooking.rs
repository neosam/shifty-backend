//! REST-Layer fuer die Rebooking-Reconciliation-Domain (Phase 55, 55-03).
//!
//! Route-Set D-55-B (Modell B), aufgeteilt in zwei Nests:
//! - Nest `/rebooking`:
//!   - POST `/manual` → `create_manual_rebooking` (F3, one-shot)
//! - Nest `/rebooking-suggestions`:
//!   - POST `/` → `create_rebooking_suggestion` (F5-Suggest/Claim — bewusste
//!     Mutation, erzeugt den pending-Batch und liefert die IST/DANN-Vorschau)
//!   - POST `/{id}/approve` → `approve_rebooking_suggestion`
//!   - POST `/{id}/reject` → `reject_rebooking_suggestion`
//!
//! Alle Handler sind duenne Wrapper: sie mappen DTO ↔ Domain, dispatchen ueber
//! `rest_state.rebooking_reconciliation_service()` und delegieren das
//! Status-Mapping (403/409/422) AUSSCHLIESSLICH an den zentralen
//! `error_handler` (kein manuelles Status-Mapping im Handler). Die HR-
//! Autorisierung liegt im Service (55-02, T-55-10); UNIQUE-Slot-Kollision und
//! Double-Approve-Race werden vom Service als `EntityAlreadyExists`
//! signalisiert und zentral → HTTP 409.

use std::collections::HashMap;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    response::Response,
    routing::post,
    Extension, Json, Router,
};
use rest_types::rebooking::{
    BackfillPreviewTO, BackfillRequestTO, ManualRebookingRequestTO, RebookingApproveRequestTO,
    RebookingSuggestRequestTO, RebookingSuggestionTO,
};
use service::rebooking_reconciliation::RebookingReconciliationService;
use shifty_utils::ShiftyWeek;
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestError, RestStateDef};

/// Router fuer den `/admin/rebooking`-Nest (F4 Backfill, HR-Admin-gated).
pub fn generate_admin_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/backfill", post(trigger_backfill::<RestState>))
}

/// Parst eine ISO-Woche im Format `YYYY-Www` (z.B. `2026-W20`) und validiert
/// sie gegen einen gueltigen ISO-Wochen-Bereich. Malformed Input (falsches
/// Muster, nicht-numerisch, Woche 0 oder groesser als `weeks_in_year`) →
/// `RestError::BadRequest` (→ 400) BEVOR der Service laeuft (T-56-11 DoS-Bound:
/// keine ressourcen-hungrige Iteration auf Basis von Muell-Input).
fn parse_iso_week(raw: &str) -> Result<ShiftyWeek, RestError> {
    let bad = || RestError::BadRequest(format!("invalid from-week '{raw}', expected YYYY-Www"));

    let (year_str, week_str) = raw.split_once("-W").ok_or_else(bad)?;
    if year_str.len() != 4 {
        return Err(bad());
    }
    let year: u32 = year_str.parse().map_err(|_| bad())?;
    let week: u8 = week_str.parse().map_err(|_| bad())?;

    // Woche muss 1..=weeks_in_year(year) sein — sonst Muster gueltig, Wert aber
    // ausserhalb des ISO-Kalenders (z.B. `2026-W99`, `2026-W00`).
    let weeks_in_year = time::util::weeks_in_year(year as i32);
    if week < 1 || week > weeks_in_year {
        return Err(bad());
    }

    Ok(ShiftyWeek { year, week })
}

/// Router fuer den `/rebooking`-Nest (F3 one-shot).
pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/manual", post(create_manual_rebooking::<RestState>))
}

/// Router fuer den `/rebooking-suggestions`-Nest (F5-Modell-B Lifecycle).
pub fn generate_suggestions_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", post(create_rebooking_suggestion::<RestState>))
        .route("/{id}/approve", post(approve_rebooking_suggestion::<RestState>))
        .route("/{id}/reject", post(reject_rebooking_suggestion::<RestState>))
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/backfill",
    tags = ["Rebooking"],
    params(
        ("from" = String, Query, description = "ISO-Woche im Format YYYY-Www (z.B. 2026-W20)"),
        ("dry_run" = bool, Query, description = "true = Vorschau (kein Write); false = Apply"),
        ("page" = Option<usize>, Query, description = "1-basierte Seite der Kandidatenliste (Default 1)"),
        ("per_page" = Option<usize>, Query, description = "Kandidaten pro Seite (Default: alle)"),
    ),
    responses(
        (status = 200, description = "Backfill-Vorschau (dry_run) bzw. Apply-Ergebnis", body = BackfillPreviewTO),
        (status = 400, description = "Malformed from-week (kein gueltiges YYYY-Www)"),
        (status = 403, description = "Forbidden (nicht HR-Admin)"),
    ),
)]
pub async fn trigger_backfill<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    error_handler(
        (async {
            // from-week parsen + validieren BEVOR der Service laeuft (T-56-11).
            let from_raw = params
                .get("from")
                .ok_or_else(|| RestError::BadRequest("from missing".to_string()))?;
            let from_week = parse_iso_week(from_raw)?;

            // dry_run: default false (Apply nur explizit); jeder andere Wert als
            // "true"/"false" → 400.
            let dry_run = match params.get("dry_run").map(String::as_str) {
                None => false,
                Some("true") => true,
                Some("false") => false,
                Some(other) => {
                    return Err(RestError::BadRequest(format!(
                        "invalid dry_run '{other}', expected true|false"
                    )))
                }
            };

            // Optionale Pagination-Bounds (D-05: multi-year Backfills nicht in
            // einer Response abladen). Ungueltige Zahlen → 400.
            let page: usize = params
                .get("page")
                .map(|p| p.parse::<usize>())
                .transpose()
                .map_err(|_| RestError::BadRequest("invalid page".to_string()))?
                .unwrap_or(1)
                .max(1);
            let per_page: Option<usize> = params
                .get("per_page")
                .map(|p| p.parse::<usize>())
                .transpose()
                .map_err(|_| RestError::BadRequest("invalid per_page".to_string()))?;

            // Delegation an den Service (HR-Admin-Gate + Upper-Bound-Clamp +
            // dry-run/apply Write-Path liegen dort — kein Business-Logic im REST).
            let items = rest_state
                .rebooking_reconciliation_service()
                .backfill_range(from_week, dry_run, context.into(), None)
                .await?;

            // total_candidates bleibt die ungeschnittene Gesamtzahl; nur die
            // items-Liste wird paginiert.
            let mut to = BackfillPreviewTO::from_domain(&items, dry_run);
            if let Some(per_page) = per_page {
                let start = page.saturating_sub(1).saturating_mul(per_page);
                to.items = to.items.into_iter().skip(start).take(per_page).collect();
            }

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
    path = "/manual",
    tags = ["Rebooking"],
    request_body = ManualRebookingRequestTO,
    responses(
        (status = 204, description = "Manuelle Umbuchung durchgefuehrt (F3 one-shot)"),
        (status = 403, description = "Forbidden (nicht HR)"),
        (status = 409, description = "UNIQUE-Slot-Kollision (Woche bereits belegt)"),
        (status = 422, description = "Validierungsfehler"),
    ),
)]
pub async fn create_manual_rebooking<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(body): Json<ManualRebookingRequestTO>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .rebooking_reconciliation_service()
                .rebook_manual(
                    body.sales_person_id,
                    body.direction.into(),
                    body.hours,
                    body.iso_year,
                    body.iso_week,
                    context.into(),
                    None,
                )
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["Rebooking"],
    request_body = RebookingSuggestRequestTO,
    responses(
        (status = 200, description = "Pending-Batch geclaimt (idempotent); backend-computed IST/DANN-Vorschau", body = RebookingSuggestionTO),
        (status = 403, description = "Forbidden (nicht HR)"),
        (status = 409, description = "Slot bereits entschieden (approved|rejected)"),
        (status = 422, description = "Validierungsfehler"),
    ),
)]
pub async fn create_rebooking_suggestion<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(body): Json<RebookingSuggestRequestTO>,
) -> Response {
    error_handler(
        (async {
            let suggestion = rest_state
                .rebooking_reconciliation_service()
                .suggest(
                    body.sales_person_id,
                    body.iso_year,
                    body.iso_week,
                    context.into(),
                    None,
                )
                .await?;
            let to = RebookingSuggestionTO::from_domain(&suggestion, body.sales_person_id);
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
    path = "/{id}/approve",
    tags = ["Rebooking"],
    params(("id", description = "Rebooking-Batch id (pending)")),
    request_body = RebookingApproveRequestTO,
    responses(
        (status = 204, description = "Vorschlag genehmigt; -N/+N-Pair geschrieben"),
        (status = 403, description = "Forbidden (nicht HR)"),
        (status = 409, description = "Double-Approve-Race-Verlierer (bereits entschieden)"),
        (status = 422, description = "Validierungsfehler"),
    ),
)]
pub async fn approve_rebooking_suggestion<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
    Json(body): Json<RebookingApproveRequestTO>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .rebooking_reconciliation_service()
                .approve(id, body.hours, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/{id}/reject",
    tags = ["Rebooking"],
    params(("id", description = "Rebooking-Batch id (pending)")),
    responses(
        (status = 204, description = "Vorschlag abgelehnt; kein Pair"),
        (status = 403, description = "Forbidden (nicht HR)"),
        (status = 409, description = "bereits entschieden"),
    ),
)]
pub async fn reject_rebooking_suggestion<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .rebooking_reconciliation_service()
                .reject(id, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        trigger_backfill,
        create_manual_rebooking,
        create_rebooking_suggestion,
        approve_rebooking_suggestion,
        reject_rebooking_suggestion,
    ),
    components(schemas(
        ManualRebookingRequestTO,
        RebookingSuggestRequestTO,
        RebookingApproveRequestTO,
        RebookingSuggestionTO,
        rest_types::rebooking::RebookingDirectionTO,
        BackfillRequestTO,
        BackfillPreviewTO,
        rest_types::rebooking::BackfillPreviewItemTO,
    )),
    tags(
        (name = "Rebooking", description = "Manuelle Umbuchung (F3) + HR-Alert-Vorschlaege (F5, Modell B)"),
    ),
)]
pub struct RebookingApiDoc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_iso_week_accepts_valid_yyyy_www() {
        assert_eq!(
            parse_iso_week("2026-W20").unwrap(),
            ShiftyWeek { year: 2026, week: 20 }
        );
        // W01 (single-digit padded) and the last week of a 53-week year.
        assert_eq!(
            parse_iso_week("2026-W01").unwrap(),
            ShiftyWeek { year: 2026, week: 1 }
        );
        // 2020 is an ISO 53-week year.
        assert_eq!(
            parse_iso_week("2020-W53").unwrap(),
            ShiftyWeek { year: 2020, week: 53 }
        );
    }

    #[test]
    fn parse_iso_week_rejects_malformed_input() {
        // T-56-11: every malformed shape must be a BadRequest (→ 400) BEFORE any
        // service iteration — never an unbounded / panicking parse.
        for bad in [
            "garbage",       // no separator
            "2026W20",       // missing -W
            "2026-20",       // missing W
            "26-W20",        // year not 4 digits
            "2026-W",        // empty week
            "2026-W00",      // week 0 (below ISO range)
            "2026-W99",      // week beyond weeks_in_year
            "2025-W53",      // 2025 has only 52 ISO weeks
            "2026-Wxx",      // non-numeric week
            "abcd-W20",      // non-numeric year
            "",              // empty
        ] {
            assert!(
                parse_iso_week(bad).is_err(),
                "expected '{bad}' to be rejected as malformed"
            );
        }
    }
}
