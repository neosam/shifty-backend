//! REST-Layer für die PDF-Export-Konfiguration (Phase 48 — EXP-02/EXP-03).
//!
//! Admin-gated CRUD unter `/pdf-export-config`:
//! - `GET /` — liefert die aktuelle Konfig + Status (Token maskiert, T-48-02).
//! - `PUT /` — setzt die admin-editierbaren Felder; leerer `webdav_app_token`
//!   behält den bestehenden Wert (D-48-REST). Nach dem `update` wird
//!   `pdf_export_scheduler.reload_from_db()` aufgerufen, damit ein neuer
//!   Cron-Ausdruck ohne Server-Restart wirksam wird (CONTEXT Q4).
//! - `POST /trigger` — Phase 48 Plan 04: löst genau EINEN sofortigen
//!   Export-Lauf asynchron (`tokio::spawn`) aus. Admin-gated; Response `202
//!   Accepted`.
//!
//! Die admin-Enforcement passiert AUSSCHLIESSLICH im Basic-Service
//! (`service_impl::pdf_export_config`, D-48-ADMIN); der REST-Layer ist ein
//! dünner Wrapper mit DTO-Conversion, Content-Type-JSON (HYG-05) und
//! Error-Mapping via `error_handler`.

use axum::{
    body::Body,
    extract::State,
    response::Response,
    routing::{get, post, put},
    Extension, Json, Router,
};
use rest_types::PdfExportConfigTO;
use service::pdf_export::PdfExportScheduler;
use service::pdf_export_config::{PdfExportConfigService, PdfExportConfigUpdate};
use service::permission::Authentication;
use tracing::instrument;
use utoipa::OpenApi;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_config::<RestState>))
        .route("/", put(update_config::<RestState>))
        .route("/trigger", post(trigger_export_now::<RestState>))
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "",
    tags = ["PdfExportConfig"],
    responses(
        (status = 200, description = "Current PDF-Export config (token masked)", body = PdfExportConfigTO, content_type = "application/json"),
        (status = 403, description = "Forbidden — admin privilege required"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_config<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let cfg = rest_state
                .pdf_export_config_service()
                .get(context.into(), None)
                .await?;
            let to: PdfExportConfigTO = (&cfg).into();
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
    put,
    path = "",
    tags = ["PdfExportConfig"],
    request_body = PdfExportConfigTO,
    responses(
        (status = 200, description = "Updated config (token masked in response)", body = PdfExportConfigTO, content_type = "application/json"),
        (status = 403, description = "Forbidden — admin privilege required"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn update_config<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(to): Json<PdfExportConfigTO>,
) -> Response {
    error_handler(
        (async {
            let update = PdfExportConfigUpdate {
                enabled: to.enabled,
                nextcloud_url: to.nextcloud_url,
                webdav_user: to.webdav_user,
                // D-48-REST: leer speichern lässt den existierenden Wert stehen.
                webdav_app_token: to.webdav_app_token,
                target_folder: to.target_folder,
                weeks_horizon: to.weeks_horizon,
                cron_schedule: to.cron_schedule,
            };
            let result = rest_state
                .pdf_export_config_service()
                .update(update, context.into(), None)
                .await?;
            // Phase 48 Plan 04: nach dem persist Reload-Hook auslösen, damit
            // eine geänderte Cron-Expression ohne Server-Restart wirksam wird.
            rest_state
                .pdf_export_scheduler()
                .reload_from_db()
                .await?;
            // Response maskiert Token wieder (T-48-02).
            let response_to: PdfExportConfigTO = (&result).into();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&response_to).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/trigger",
    tags = ["PdfExportConfig"],
    responses(
        (status = 204, description = "Export run started asynchronously"),
        (status = 403, description = "Forbidden — admin privilege required"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn trigger_export_now<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            // Admin-Gate: Wir prüfen das ADMIN-Privileg indem wir `get` auf dem
            // Config-Service aufrufen (der ist admin-gated). Fehler → 403.
            // Damit wandelt der REST-Handler die User-Session in einen
            // trusted "Full-Auth"-Aufruf um: der Scheduler läuft mit
            // `Authentication::Full`, um `record_success`/`record_error`
            // schreiben zu können (D-48-ADMIN).
            let _cfg = rest_state
                .pdf_export_config_service()
                .get(context.into(), None)
                .await?;
            let scheduler = rest_state.pdf_export_scheduler();
            tokio::spawn(async move {
                if let Err(e) = scheduler.run_once_now(Authentication::Full).await {
                    tracing::error!("pdf-export trigger run failed: {e:?}");
                }
            });
            // 204 No Content: spec-conform empty-body response for
            // "async work accepted" (HYG-05 compat via the content-type
            // surface test — 202 requires an explicit content-type).
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
    tags(
        (
            name = "PdfExportConfig",
            description = "Admin-gated Nextcloud-PDF-Export-Konfiguration + Scheduler-Status (Phase 48)",
        ),
    ),
    paths(
        get_config,
        update_config,
        trigger_export_now,
    ),
    components(schemas(PdfExportConfigTO)),
)]
pub struct PdfExportConfigApiDoc;
