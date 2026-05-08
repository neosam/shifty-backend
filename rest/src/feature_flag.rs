//! Phase 8 Plan 08-07 Gap-Closure (Task 2) — Feature-Flag REST-Layer.
//!
//! Eine einzige Route: `GET /feature-flag/{key}`.
//! - Auth-only readable (jeder authentifizierte User darf lesen — unauth
//!   liefert 401 bereits in der globalen `forbid_unauthenticated`-Middleware).
//! - Bei unbekanntem Key liefert das Service `Ok(false)`; der Handler übersetzt
//!   das in `200 OK` mit `{ "key": ..., "enabled": false, "description": null }`
//!   (fail-safe — kein 404, weil das Frontend einfach erkennt, dass das Flag
//!   "as if disabled" ist).
//!
//! Service-Read-API ist `is_enabled(key)` — eine separate `find_flag`-Methode
//! existiert nicht. Wir benutzen direkt das DAO über den Service-Trait nicht;
//! stattdessen liefert der Endpoint nur Key + Enabled, ohne `description`,
//! weil das Service-Trait `description` nicht aggregiert. Das ist akzeptabel
//! für den Cutover-Use-Case (Frontend braucht nur `enabled`).
//!
//! Permission-Enforcement liegt bei `is_enabled` selbst (Auth::Context-Pfad
//! ruft `current_user_id`-Lookup; unauth → ServiceError::Unauthorized → 401).

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::get,
    Extension, Router,
};
use rest_types::FeatureFlagTO;
use service::feature_flag::FeatureFlagService;
use tracing::instrument;
use utoipa::OpenApi;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/{key}", get(get_feature_flag_handler::<RestState>))
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{key}",
    tags = ["FeatureFlag"],
    params(("key" = String, Path, description = "Feature-Flag key, z.B. 'absence_range_source_active'")),
    responses(
        (status = 200, description = "Feature-Flag-Wert. Bei unbekanntem Key: enabled = false (fail-safe).", body = FeatureFlagTO),
        (status = 401, description = "Unauthenticated."),
    ),
)]
pub async fn get_feature_flag_handler<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(key): Path<String>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.feature_flag_service();
            let enabled = svc.is_enabled(&key, context.into(), None).await?;
            // Description wird vom Trait nicht zurückgegeben; das ist ok für
            // den Cutover-Use-Case (FE braucht nur `enabled`). Wenn künftig
            // eine `find_flag`-Methode am Trait erscheint, kann der Body hier
            // angereichert werden.
            let to = FeatureFlagTO {
                key,
                enabled,
                description: None,
            };
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
    paths(get_feature_flag_handler),
    components(schemas(FeatureFlagTO)),
    tags(
        (
            name = "FeatureFlag",
            description = "Read access to backend feature flags. Auth-only readable; \
                           writes require the `feature_flag_admin` privilege and \
                           are NOT exposed via REST in Phase 8.",
        ),
    ),
)]
pub struct FeatureFlagApiDoc;
