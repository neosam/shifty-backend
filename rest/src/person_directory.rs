//! Phase 62 Plan 05 (PADM-01 / PADM-08): REST-Oberflaeche des
//! Personen-Verzeichnisses — `GET /person-directory`.
//!
//! **Das Autorisierungs-Gate liegt im Service, nicht hier (D-62-08, T-62-13).**
//! `PersonDirectoryServiceImpl::list` verlangt das Admin-Privileg als erste
//! Anweisung, vor jedem Datenbankzugriff. Der Handler prueft bewusst **nicht**
//! zusaetzlich: zwei Gate-Stellen sind zwei Wahrheiten, die auseinanderlaufen
//! koennen, und die schwaechere von beiden waere dann die Luecke. Der Handler
//! reicht den Session-Kontext durch und laesst `error_handler` den
//! `ServiceError::Forbidden` auf 403 abbilden.
//!
//! **Der Praefix `/person-directory` ist ein Vertrag ueber vier Stellen.** Er
//! steht im `.nest(...)`-Aufruf und im `ApiDoc`-Nest (beides `rest/src/lib.rs`),
//! in der Aufruf-URL des Frontends (`shifty-dioxus/src/api.rs`) und im
//! `[[web.proxy]]`-Eintrag (`shifty-dioxus/Dioxus.toml`). Alle vier muessen
//! zeichengleich bleiben; ein fehlender Proxy-Eintrag hat dieses Repo bereits
//! dreimal einen 404 im Dev-Server gekostet (Roadmap-Constraint 6, T-62-14).
//!
//! **Fat Backend (PADM-06).** Der Handler rechnet nichts nach: `display_name`
//! und `kind` kommen fertig aus dem Service, das Mapping auf `PersonEntryTO`
//! ist rein feldweise (62-03).

use std::sync::Arc;

use axum::{body::Body, extract::State, response::Response, routing::get, Extension, Router};
use rest_types::{PersonEntryTO, PersonKindTO};
use tracing::instrument;
use utoipa::OpenApi;

use crate::{error_handler, Context, RestStateDef};
use service::person_directory::PersonDirectoryService;

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/", get(get_person_directory::<RestState>))
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/",
    tags = ["PersonDirectory"],
    responses(
        (status = 200, description = "The full person directory: linked persons, user-only accounts and sales-person-only rows side by side, sorted case-insensitively by display name", body = [PersonEntryTO]),
        (status = 403, description = "Not an admin"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_person_directory<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let entries: Arc<[PersonEntryTO]> = rest_state
                .person_directory_service()
                .list(context.into(), None)
                .await?
                .iter()
                .map(PersonEntryTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::from(serde_json::to_string(&entries).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "PersonDirectory", description = "Admin-gated person directory (PADM-01)"),
    ),
    paths(
        get_person_directory,
    ),
    components(
        schemas(
            PersonEntryTO,
            PersonKindTO,
        ),
    ),
)]
pub struct PersonDirectoryApiDoc;
