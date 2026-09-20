//! Phase 63 Plan 06 (PADM-02 … PADM-05, PADM-07, PADM-08): REST-Oberflaeche
//! der Personen-Detailansicht — die sieben Operationen unter `/person`.
//!
//! **Das Autorisierungs-Gate liegt im Service, nicht hier (D-62-08, T-63-06-A).**
//! Jede der sieben Methoden von `PersonDetailServiceImpl` verlangt das
//! Admin-Privileg als erste Anweisung, vor jeder Validierung und vor jedem
//! Datenbankzugriff. Die Handler pruefen bewusst **nicht** zusaetzlich: zwei
//! Gate-Stellen sind zwei Wahrheiten, die auseinanderlaufen koennen, und die
//! schwaechere von beiden waere dann die Luecke. Die Handler reichen den
//! Session-Kontext durch; die zentrale Fehler-Abbildung von `rest/src/lib.rs`
//! uebersetzt `ServiceError::Forbidden` in 403. Kein Handler bildet einen
//! Status selbst ab.
//!
//! **Der Praefix `/person` ist ein Vertrag ueber vier Stellen.** Er steht
//! (1) im `.nest(...)`-Aufruf und (2) im `ApiDoc`-Nest — beides
//! `rest/src/lib.rs` —, (3) in der Aufruf-URL des Frontends
//! (`shifty-dioxus/src/api.rs`) und (4) im `[[web.proxy]]`-Eintrag
//! (`shifty-dioxus/Dioxus.toml`). Alle vier muessen zeichengleich bleiben; ein
//! fehlender Proxy-Eintrag hat dieses Repo bereits dreimal einen 404 im
//! Dev-Server gekostet (v1.6 `/toggle`, v1.8 `/vacation-entitlement-offset`,
//! v1.10 `/special-days`; Roadmap-Constraint 6, T-63-06-D). Kein `cargo test`
//! faengt diese Fehlerklasse — der Nachweis gehoert in den manuellen Durchgang.
//!
//! **Reihenfolge im Proxy beachten.** Der Dev-Server-Proxy matcht
//! praefix-literal, und `/person` ist ein Praefix von `/person-directory`. Der
//! generische Eintrag steht deshalb in `Dioxus.toml` als **letzter** der
//! beiden; stuende er davor, finge er die Verzeichnis-Anfragen mit ab.
//!
//! **Fat Backend (PADM-06).** Die Handler rechnen nichts nach: `kind` und
//! `display_name` kommen fertig aus dem Service, das Mapping auf
//! `PersonEntryTO` ist rein feldweise (62-03). Insbesondere leitet kein Handler
//! aus der Leerheit von `username` oder `sales_person_id` eine Auspraegung ab.
//!
//! **Kein neuer Verknuepfungs-Schreibpfad.** `POST`/`DELETE
//! /sales-person/{id}/user` aus Phase 61 bleiben die einzigen Endpoints, die
//! eine bestehende Sales Person mit einem bestehenden Login verbinden bzw.
//! trennen. Die Kandidatenlisten hier blenden bereits verknuepfte Ziele aus —
//! das ist Bequemlichkeit, nicht die Grenze: der Phase-61-Pfad lehnt ein
//! besetztes Ziel unabhaengig davon mit 409 ab (D-61-04).

use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::{delete, get, post};
use axum::{Extension, Json, Router};
use rest_types::{
    AddPersonFacetTO, CreatePersonEntryTO, CreatedPersonEntryTO, PersonEntryTO, PersonFacetTO,
    PersonKindTO, SalesPersonTO, UserTO,
};
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};
use service::person_detail::PersonDetailService;

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    // Die beiden literalen Pfade stehen vor dem Parameter-Pfad, damit der
    // Router-Aufbau lesbar bleibt — `/candidates/...` und `/by-sales-person/...`
    // koennten sonst wie Spezialfaelle von `/{person_id}` wirken.
    Router::new()
        .route(
            "/candidates/sales-persons",
            get(get_unlinked_sales_persons::<RestState>),
        )
        .route("/candidates/users", get(get_unlinked_users::<RestState>))
        .route(
            "/by-sales-person/{sales_person_id}",
            get(get_person_by_sales_person::<RestState>),
        )
        .route("/", post(create_person_entry::<RestState>))
        .route("/{person_id}", get(get_person_detail::<RestState>))
        .route("/{person_id}/facet", post(add_person_facet::<RestState>))
        .route(
            "/{person_id}/facet/{facet}",
            delete(remove_person_facet::<RestState>),
        )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/candidates/sales-persons",
    tags = ["PersonDetail"],
    description = "Sales persons that do not carry a user account yet — the picker behind \"add login\" (PADM-04). Already linked sales persons are omitted for convenience only; POST /sales-person/{id}/user rejects an occupied target with 409 regardless (D-61-04).",
    responses(
        (status = 200, description = "Sales persons without a linked user account", body = [SalesPersonTO]),
        (status = 403, description = "Not an admin"),
        (status = 404, description = "Not applicable — listed for symmetry with the other read operations"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_unlinked_sales_persons<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let candidates: Arc<[SalesPersonTO]> = rest_state
                .person_detail_service()
                .unlinked_sales_person_candidates(context.into(), None)
                .await?
                .iter()
                .map(SalesPersonTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&candidates).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/candidates/users",
    tags = ["PersonDetail"],
    description = "Usernames that do not carry a sales person yet — the picker behind \"add person\" (PADM-04). Already linked users are omitted for convenience only; POST /sales-person/{id}/user rejects an occupied target with 409 regardless (D-61-04).",
    responses(
        (status = 200, description = "Usernames without a linked sales person", body = [UserTO]),
        (status = 403, description = "Not an admin"),
        (status = 404, description = "Not applicable — listed for symmetry with the other read operations"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_unlinked_users<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let candidates: Arc<[UserTO]> = rest_state
                .person_detail_service()
                .unlinked_user_candidates(context.into(), None)
                .await?
                .iter()
                .map(|username| UserTO {
                    name: username.to_string(),
                })
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&candidates).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/by-sales-person/{sales_person_id}",
    tags = ["PersonDetail"],
    description = "The very same directory row as GET /person/{person_id}, resolved through the sales person id (PADM-04). Needed after linking an existing sales person: the person_id changed during the merge (D-61-03), so the detail page can only find its way back through the sales person id.",
    params(
        ("sales_person_id", description = "Sales person ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    responses(
        (status = 200, description = "The directory row of the person carrying this sales person", body = PersonEntryTO),
        (status = 403, description = "Not an admin"),
        (status = 404, description = "Sales person unknown, not resolvable to a person, or the person is invisible in the directory (D-62-04)"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_person_by_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let entry = rest_state
                .person_detail_service()
                .get_detail_by_sales_person(sales_person_id, context.into(), None)
                .await?;
            let entry = PersonEntryTO::from(&entry);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&entry).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["PersonDetail"],
    description = "Creates a directory entry with exactly the selected facets (PADM-02). User account and sales person are created in ONE transaction — if either creation fails, neither exists afterwards and no new person row remains.",
    request_body (
        content = CreatePersonEntryTO,
        content_type = "application/json"
    ),
    responses(
        (status = 201, description = "The new entry — the response carries the person_id the dialog navigates to", body = CreatedPersonEntryTO),
        (status = 403, description = "Not an admin"),
        (status = 422, description = "Neither toggle set, or a mandatory name is missing/blank. Enforced server-side: the disabled submit button in the frontend is convenience, not the boundary (D-63-07, ASVS V5)"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn create_person_entry<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(request): Json<CreatePersonEntryTO>,
) -> Response {
    error_handler(
        (async {
            let person_id = rest_state
                .person_detail_service()
                .create_combined(&(&request).into(), context.into(), None)
                .await?;
            let created = CreatedPersonEntryTO { person_id };
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&created).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{person_id}",
    tags = ["PersonDetail"],
    description = "The directory row of exactly one person (PADM-05, PADM-06). Carries the same fields as the list row from phase 62, with kind and display_name already derived in the backend. The four sections of the page load their own, already running endpoints.",
    params(
        ("person_id", description = "Person ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    responses(
        (status = 200, description = "The directory row", body = PersonEntryTO),
        (status = 403, description = "Not an admin"),
        (status = 404, description = "Person unknown or invisible in the directory — never a half-empty answer (D-62-04)"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_person_detail<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let entry = rest_state
                .person_detail_service()
                .get_detail(person_id, context.into(), None)
                .await?;
            let entry = PersonEntryTO::from(&entry);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&entry).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/{person_id}/facet",
    tags = ["PersonDetail"],
    description = "Adds the missing facet to an existing entry (PADM-03). The response carries the RESULTING person_id, which may differ from the path parameter: adding a sales person to a user-only entry merges both identities and the sales person wins (D-61-03) — callers must follow the returned id.",
    params(
        ("person_id", description = "Person ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    request_body (
        content = AddPersonFacetTO,
        content_type = "application/json"
    ),
    responses(
        (status = 200, description = "The resulting entry — use this person_id, not the one from the path", body = CreatedPersonEntryTO),
        (status = 403, description = "Not an admin"),
        (status = 404, description = "Person unknown or invisible in the directory"),
        (status = 409, description = "The requested side is already occupied. Same contract as POST /sales-person/{id}/user in phase 61 (D-61-04): hiding an occupied target from the candidate list is convenience, this rejection is the boundary"),
        (status = 422, description = "The mandatory name for the requested side is missing or blank"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn add_person_facet<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(person_id): Path<Uuid>,
    Json(request): Json<AddPersonFacetTO>,
) -> Response {
    error_handler(
        (async {
            let resulting_person_id = rest_state
                .person_detail_service()
                .add_facet(
                    person_id,
                    (&request.facet).into(),
                    request.display_name.clone(),
                    request.username.clone(),
                    context.into(),
                    None,
                )
                .await?;
            let created = CreatedPersonEntryTO {
                person_id: resulting_person_id,
            };
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&created).unwrap()))
                .unwrap())
        })
        .await,
    )
}

/// Die Facet-Kennung steht bewusst im **Pfad** und nicht im Body: `DELETE`
/// arbeitet in diesem Repo durchgaengig ohne Body (`/sales-person/{id}/user`,
/// `/sales-person/unavailable/{id}`, `/extra-hours/{id}`). Die Drahtform des
/// Parameters ist `snake_case` — `user` bzw. `sales_person`, dieselbe
/// Konvention wie bei `PersonKindTO` (63-04).
#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/{person_id}/facet/{facet}",
    tags = ["PersonDetail"],
    description = "Removes EXACTLY ONE facet of an entry (PADM-07). The two sides are deliberately asymmetric (D-63-05): removing the login is a hard delete (roles and open invitations cascade), removing the sales person is a soft delete (shift plan history and write-once billing snapshots stay intact). If the person loses its last living carrier, the person row is removed in the same transaction (D-63-01). There is no bulk button.",
    params(
        ("person_id", description = "Person ID", example = "123e4567-e89b-12d3-a456-426614174000"),
        ("facet", description = "Which facet to remove: `user` or `sales_person`", example = "user"),
    ),
    responses(
        (status = 204, description = "Facet removed"),
        (status = 403, description = "Not an admin"),
        (status = 404, description = "Person unknown, or the requested side is not present"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn remove_person_facet<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((person_id, facet)): Path<(Uuid, PersonFacetTO)>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .person_detail_service()
                .remove_facet(person_id, (&facet).into(), context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "PersonDetail", description = "Admin-gated person detail view and facet actions (PADM-02 … PADM-05, PADM-07)"),
    ),
    paths(
        get_unlinked_sales_persons,
        get_unlinked_users,
        get_person_by_sales_person,
        create_person_entry,
        get_person_detail,
        add_person_facet,
        remove_person_facet,
    ),
    components(
        schemas(
            PersonEntryTO,
            PersonKindTO,
            CreatePersonEntryTO,
            CreatedPersonEntryTO,
            PersonFacetTO,
            AddPersonFacetTO,
            SalesPersonTO,
            UserTO,
        ),
    ),
)]
pub struct PersonDetailApiDoc;
