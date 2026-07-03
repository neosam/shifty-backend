//! REST-Layer für den On-Demand-PDF-Download der Wochen-Ansicht
//! (Phase 49 — PDF-03/PDF-04/PDF-05).
//!
//! Route:
//! - `GET /shiftplan/{shiftplan_id}/{year}/{week}/pdf` — liefert
//!   `application/pdf`-Bytes mit
//!   `Content-Disposition: attachment; filename="schichtplan-{JJJJ}-KW{NN:02}.pdf"`.
//!
//! ## 409-Pre-Check (D-49-03, PDF-04)
//!
//! Der Handler ruft VOR dem Rendering `WeekStatusService::get_week_status` auf.
//! Bei Status NICHT in `{Planned, Locked}` returned er direkt einen 409-Conflict
//! mit `application/json`-Body `{"error":"week-not-releasable"}` — ohne die
//! (potenziell teure) `render_week_pdf`-Pipeline anzustoßen.
//!
//! Das Service-interne Gate (`PdfShiftplanServiceImpl::render_week_pdf`, Wave 1)
//! bleibt als Defense-in-Depth aktiv — es fängt Race-Windows (<1ms zwischen
//! Pre-Check und Rendering) und Direct-Impl-Aufrufer (Scheduler, Plan 03) ab
//! und mapped dann zu 422 via `ServiceError::ValidationError`.
//!
//! ## Auth (D-49-07, PDF-05)
//!
//! Kein Admin-Gate. Der Handler nutzt nur die bestehende
//! `forbid_unauthenticated`-Middleware (rest/src/lib.rs) — 401 bei fehlender
//! Auth, 200 für alle authentifizierten Employee-Rollen. Der `context` wird
//! als `Authentication<Context>` an alle Service-Calls weitergereicht;
//! niemals `Authentication::Full`.

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::get,
    Extension, Router,
};
use service::{
    pdf_shiftplan::PdfShiftplanService,
    week_status::{WeekStatus, WeekStatusService},
};
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route(
        "/{shiftplan_id}/{year}/{week}/pdf",
        get(download_week_pdf::<RestState>),
    )
}

/// Reine Gate-Prüfung: darf für diesen Status ein PDF ausgeliefert werden?
///
/// Freigegeben sind nur `Planned` und `Locked` (D-49-06). Ausgesondert:
/// `Unset` (noch keine Woche gesetzt) und `InPlanning` (WIP, nicht release-
/// fähig). Als freie Funktion extrahiert, damit die Regel direkt und ohne
/// Router-Setup unit-testbar ist.
pub(crate) fn week_status_allows_download(status: &WeekStatus) -> bool {
    matches!(status, WeekStatus::Planned | WeekStatus::Locked)
}

/// Baut die 409-Antwort für einen nicht release-fähigen Wochen-Status.
///
/// D-49-03 + Handler-Level-Pre-Check: JSON-Body mit stabilem Fehler-Code
/// `week-not-releasable`, den das FE (Plan 04) für lokalisierte User-Meldungen
/// switchen kann. Content-Type explizit `application/json` (HYG-05-Konvention).
pub(crate) fn not_releasable_response() -> Response {
    Response::builder()
        .status(409)
        .header("Content-Type", "application/json")
        .body(Body::new(r#"{"error":"week-not-releasable"}"#.to_string()))
        .expect("static 409 response is well-formed")
}

/// Baut die 200-Antwort für ein gerendertes Wochen-PDF.
///
/// Setzt `Content-Type: application/pdf` und `Content-Disposition: attachment;
/// filename="schichtplan-{JJJJ}-KW{NN:02}.pdf"` (D-49-01 + PDF-03). Der
/// Filename ist rein ASCII (siehe `service::pdf_shiftplan::filename_for`),
/// keine RFC-5987-Encoding nötig.
pub(crate) fn pdf_response(bytes: Vec<u8>, year: u32, calendar_week: u8) -> Response {
    let filename = service::pdf_shiftplan::filename_for(year, calendar_week);
    Response::builder()
        .status(200)
        .header("Content-Type", "application/pdf")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(bytes))
        .expect("static headers + ascii filename produce a valid response")
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{shiftplan_id}/{year}/{week}/pdf",
    tags = ["PdfShiftplan"],
    params(
        ("shiftplan_id", description = "Shiftplan UUID"),
        ("year", description = "ISO year", example = "2026"),
        ("week", description = "ISO calendar week (1..=53)", example = "27"),
    ),
    responses(
        (status = 200, description = "PDF bytes for the week", content_type = "application/pdf", body = Vec<u8>),
        (status = 401, description = "Unauthenticated"),
        (status = 404, description = "Shiftplan not found"),
        (status = 409, description = "Week not releasable (WeekStatus in {Unset, InPlanning})", content_type = "application/json"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn download_week_pdf<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((shiftplan_id, year, week)): Path<(Uuid, u32, u8)>,
) -> Response {
    // Fast-path 409-Pre-Check (D-49-03, PDF-04) — läuft VOR dem Service-Call,
    // damit der 409-Pfad nicht vom Service-`ValidationError` (→ 422) über-
    // steuert wird (RESEARCH.md Pitfall 3). Bei WeekStatus-Fehler (500) oder
    // Service-Erfolg: normal in den Handler-Body.
    let pre_check = {
        let ctx: service::permission::Authentication<Context> = context.clone().into();
        rest_state
            .week_status_service()
            .get_week_status(year, week, ctx, None)
            .await
    };
    match pre_check {
        Ok(status) if !week_status_allows_download(&status) => {
            return not_releasable_response();
        }
        Ok(_) => { /* Planned or Locked: fall through to render */ }
        Err(e) => {
            return error_handler(Err(crate::RestError::ServiceError(e)));
        }
    }

    error_handler(
        (async {
            let bytes = rest_state
                .pdf_shiftplan_service()
                .render_week_pdf(shiftplan_id, year, week, context.into(), None)
                .await?;
            Ok(pdf_response(bytes, year, week))
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (
            name = "PdfShiftplan",
            description = "On-demand PDF-Download der Wochen-Ansicht (Phase 49)",
        ),
    ),
    paths(download_week_pdf),
)]
pub struct PdfShiftplanApiDoc;

// ─── Tests ────────────────────────────────────────────────────────────────
//
// Router-/Handler-Tests via `RestStateDef` sind hier bewusst NICHT eingebaut:
// `RestStateDef` hat 37 assoc-types + 35 Accessors — eine `TestState`-Struct
// mit passenden Mock-Typen ist unverhältnismäßig groß und nicht vom Plan-
// Kern-Signal gedeckt (die 4 Verhaltensregeln des Handlers). Statt dessen
// werden die drei rein-funktionalen Kernstücke direkt getestet:
//
// 1. `week_status_allows_download` — die Status-Matrix (PDF-04 Gate).
// 2. `not_releasable_response` — 409 + JSON-Content-Type + Body-Format.
// 3. `pdf_response` — 200 + `application/pdf` + `Content-Disposition`-Format.
//
// Full-Stack-Router-Coverage (Auth-Middleware, DI-Wiring, Real-DB-Roundtrip)
// gehört in die shifty_bin-Integrationstest-Suite, wo `RestStateDef` bereits
// vom Production-Backend erfüllt wird.
#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    // ─── PDF-04: WeekStatus-Gate-Matrix ─────────────────────────────────

    #[test]
    fn week_status_planned_allows_download() {
        assert!(week_status_allows_download(&WeekStatus::Planned));
    }

    #[test]
    fn week_status_locked_allows_download() {
        assert!(week_status_allows_download(&WeekStatus::Locked));
    }

    #[test]
    fn week_status_unset_blocks_download() {
        assert!(!week_status_allows_download(&WeekStatus::Unset));
    }

    #[test]
    fn week_status_in_planning_blocks_download() {
        assert!(!week_status_allows_download(&WeekStatus::InPlanning));
    }

    // ─── D-49-03: 409-Body Format ───────────────────────────────────────

    #[tokio::test]
    async fn not_releasable_returns_409_json_with_stable_error_code() {
        let resp = not_releasable_response();
        assert_eq!(resp.status(), 409);
        assert_eq!(
            resp.headers()
                .get("Content-Type")
                .and_then(|h| h.to_str().ok()),
            Some("application/json"),
        );
        let body_bytes = resp
            .into_body()
            .collect()
            .await
            .expect("body collect")
            .to_bytes();
        let body_str = std::str::from_utf8(&body_bytes).expect("utf8");
        assert_eq!(body_str, r#"{"error":"week-not-releasable"}"#);
    }

    // ─── PDF-03: 200-Response + Content-Disposition Filename-Format ────

    #[tokio::test]
    async fn pdf_response_sets_pdf_content_type_and_filename() {
        let bytes = b"%PDF-1.7\n%dummy".to_vec();
        let resp = pdf_response(bytes.clone(), 2026, 27);
        assert_eq!(resp.status(), 200);
        assert_eq!(
            resp.headers()
                .get("Content-Type")
                .and_then(|h| h.to_str().ok()),
            Some("application/pdf"),
        );
        assert_eq!(
            resp.headers()
                .get("Content-Disposition")
                .and_then(|h| h.to_str().ok()),
            Some(r#"attachment; filename="schichtplan-2026-KW27.pdf""#),
        );
        let body_bytes = resp
            .into_body()
            .collect()
            .await
            .expect("body collect")
            .to_bytes();
        assert_eq!(body_bytes.as_ref(), bytes.as_slice());
    }

    #[tokio::test]
    async fn pdf_response_filename_uses_leading_zero_for_single_digit_weeks() {
        let resp = pdf_response(b"%PDF-".to_vec(), 2026, 3);
        assert_eq!(
            resp.headers()
                .get("Content-Disposition")
                .and_then(|h| h.to_str().ok()),
            Some(r#"attachment; filename="schichtplan-2026-KW03.pdf""#),
        );
    }

    #[tokio::test]
    async fn pdf_response_filename_handles_week_52() {
        let resp = pdf_response(b"%PDF-".to_vec(), 2025, 52);
        assert_eq!(
            resp.headers()
                .get("Content-Disposition")
                .and_then(|h| h.to_str().ok()),
            Some(r#"attachment; filename="schichtplan-2025-KW52.pdf""#),
        );
    }
}
