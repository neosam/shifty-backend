//! REST-Layer fuer die Absence-Domain (Phase 1 — Range-based absence).
//!
//! Sechs Routen unter `/absence-period` (Bindestrich, D-01):
//! POST `/`, GET `/`, GET `/{id}`, PUT `/{id}`, DELETE `/{id}`,
//! GET `/by-sales-person/{sales_person_id}`. Jeder Handler traegt
//! `#[utoipa::path]` (CC-06) + `#[instrument(skip(rest_state))]`.
//!
//! PUT-Handler ueberschreibt `entity.id = path_id` (path-id wins). Die
//! Service-Layer-Verifikation (Permission, Self-Overlap, Range) wird in
//! `service::absence::AbsenceService` durchgefuehrt; der REST-Layer ist
//! ein duenner Wrapper mit DTO-Conversion und Error-Mapping via
//! `error_handler`. Alle Handler dispatchen ueber `rest_state.absence_service()`
//! gemaess `RestStateDef`-Trait.
//!
//! Phase 8.5 (Plan 04): GET / + GET /by-sales-person/{id} geben jetzt
//! `AbsenceListWithProjectionTO` zurueck — Ranges + lebende Stunden-Marker
//! (Vacation/SickLeave/UnpaidLeave) ehrlich am when-Datum (D-07, kein Range-Raten).

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use rest_types::{
    AbsenceCategoryTO, AbsencePeriodCreateResultTO, AbsencePeriodTO, AbsenceListWithProjectionTO,
    ExtraHoursMarkerTO, WarningTO,
};
use service::absence::AbsenceService;
use service::extra_hours::{ExtraHoursCategory, ExtraHoursService};
use service::sales_person::SalesPersonService;
use shifty_utils::ShiftyDate;
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

/// Konvertiert eine lebende ExtraHours-Row zu einem ExtraHoursMarkerTO.
/// Kein Range-Raten (D-07) — `when` traegt raw `date_time.date()`.
fn map_to_marker(
    eh: &service::extra_hours::ExtraHours,
    person_name: std::sync::Arc<str>,
) -> ExtraHoursMarkerTO {
    ExtraHoursMarkerTO {
        extra_hours_id: eh.id,
        sales_person_id: eh.sales_person_id,
        when: eh.date_time.date(),
        amount: eh.amount,
        category: (&eh.category).into(),
        description: eh.description.clone(),
        person_name,
    }
}

/// Prueft ob eine ExtraHoursCategory zur Read-Projektion gehoert.
/// Nur {Vacation, SickLeave, UnpaidLeave} werden als Marker angezeigt.
fn is_absence_category(category: &ExtraHoursCategory) -> bool {
    matches!(
        category,
        ExtraHoursCategory::Vacation | ExtraHoursCategory::SickLeave | ExtraHoursCategory::UnpaidLeave
    )
}

/// Berechnet das Zwei-Jahres-Fenster [current_year-1, current_year+1] fuer Marker-Loads.
/// Quelle: Pitfall 7 / RESEARCH Open Question 2.
fn two_year_window() -> (ShiftyDate, ShiftyDate) {
    let current_year = time::OffsetDateTime::now_utc().year();
    let from = ShiftyDate::from(
        time::Date::from_calendar_date(current_year - 1, time::Month::January, 1)
            .expect("valid from_date"),
    );
    let to = ShiftyDate::from(
        time::Date::from_calendar_date(current_year + 1, time::Month::December, 31)
            .expect("valid to_date"),
    );
    (from, to)
}

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", post(create_absence_period::<RestState>))
        .route("/", get(get_all_absence_periods::<RestState>))
        .route("/{id}", get(get_absence_period::<RestState>))
        .route("/{id}", put(update_absence_period::<RestState>))
        .route("/{id}", delete(delete_absence_period::<RestState>))
        .route(
            "/by-sales-person/{sales_person_id}",
            get(get_absence_periods_for_sales_person::<RestState>),
        )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["Absence"],
    request_body = AbsencePeriodTO,
    responses(
        (status = 201, description = "Absence period created (with warnings if any)", body = AbsencePeriodCreateResultTO),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Validation error"),
    ),
)]
pub async fn create_absence_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(body): Json<AbsencePeriodTO>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            // Phase-3 Plan-05: Body ist jetzt der volle Wrapper-Result
            // mit Forward-Warnings (BOOK-01). Frontend rendert
            // `.warnings` als Liste; `.absence` ist die persistierte
            // AbsencePeriod (analog Phase 1).
            let result = svc.create(&(&body).into(), context.into(), None).await?;
            let to = AbsencePeriodCreateResultTO::from(&result);
            Ok(Response::builder()
                .status(201)
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
    path = "",
    tags = ["Absence"],
    responses(
        (status = 200, description = "All absence periods with living hourly markers (Vacation/SickLeave/UnpaidLeave)", body = AbsenceListWithProjectionTO),
        (status = 403, description = "Forbidden"),
    ),
)]
pub async fn get_all_absence_periods<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            let entities = svc.find_all(context.clone().into(), None).await?;
            let absence_periods: Vec<AbsencePeriodTO> =
                entities.iter().map(AbsencePeriodTO::from).collect();

            // Lade alle Personen (HR-View — find_all enforced bereits hr-Gate).
            let persons = rest_state
                .sales_person_service()
                .get_all(context.clone().into(), None)
                .await?;

            // Zwei-Jahres-Fenster fuer Marker-Loads (Pitfall 7).
            let (from_bound, to_bound) = two_year_window();

            // Fuer jede Person: lebende extra_hours laden, auf Absence-Kategorien filtern.
            let mut hourly_markers: Vec<ExtraHoursMarkerTO> = Vec::new();
            for person in persons.iter() {
                let raw = rest_state
                    .extra_hours_service()
                    .find_by_sales_person_id_and_year_range(
                        person.id,
                        from_bound,
                        to_bound,
                        context.clone().into(),
                        None,
                    )
                    .await?;
                for eh in raw.iter() {
                    if is_absence_category(&eh.category) {
                        hourly_markers.push(map_to_marker(eh, person.name.clone()));
                    }
                }
            }

            let result = AbsenceListWithProjectionTO {
                absence_periods,
                hourly_markers,
            };
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&result).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}",
    tags = ["Absence"],
    params(("id", description = "Absence period logical id")),
    responses(
        (status = 200, description = "Absence period", body = AbsencePeriodTO),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
)]
pub async fn get_absence_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            let entity = svc.find_by_id(id, context.into(), None).await?;
            let to = AbsencePeriodTO::from(&entity);
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
    path = "/{id}",
    tags = ["Absence"],
    params(("id", description = "Absence period logical id")),
    request_body = AbsencePeriodTO,
    responses(
        (status = 200, description = "Updated absence period (with warnings if any)", body = AbsencePeriodCreateResultTO),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 409, description = "Version conflict"),
        (status = 422, description = "Validation error"),
    ),
)]
pub async fn update_absence_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(absence_id): Path<Uuid>,
    Json(body): Json<AbsencePeriodTO>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            let mut entity: service::absence::AbsencePeriod = (&body).into();
            entity.id = absence_id; // path-id wins (D-01 / Pitfall guard)
            // Phase-3 Plan-05: Body ist jetzt der volle Wrapper-Result
            // mit Forward-Warnings (BOOK-01).
            let result = svc.update(&entity, context.into(), None).await?;
            let to = AbsencePeriodCreateResultTO::from(&result);
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
    delete,
    path = "/{id}",
    tags = ["Absence"],
    params(("id", description = "Absence period logical id")),
    responses(
        (status = 204, description = "Soft-deleted"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
    ),
)]
pub async fn delete_absence_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            svc.delete(id, context.into(), None).await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/by-sales-person/{sales_person_id}",
    tags = ["Absence"],
    params(("sales_person_id", description = "Sales person id")),
    responses(
        (status = 200, description = "Absence periods + living hourly markers for sales person", body = AbsenceListWithProjectionTO),
        (status = 403, description = "Forbidden"),
    ),
)]
pub async fn get_absence_periods_for_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            let entities = svc
                .find_by_sales_person(sales_person_id, context.clone().into(), None)
                .await?;
            let absence_periods: Vec<AbsencePeriodTO> =
                entities.iter().map(AbsencePeriodTO::from).collect();

            // Lade die eine Person fuer person_name im Marker.
            let person = rest_state
                .sales_person_service()
                .get(sales_person_id, context.clone().into(), None)
                .await?;

            // Zwei-Jahres-Fenster (Pitfall 7).
            let (from_bound, to_bound) = two_year_window();

            // Marker nur fuer diese Person (hr ∨ self erbt von /absences + extra_hours-Scoping).
            // KEIN Authentication::Full-Bypass — Context unveraendert durchreichen (D-06, T-8.5-04a).
            let raw = rest_state
                .extra_hours_service()
                .find_by_sales_person_id_and_year_range(
                    sales_person_id,
                    from_bound,
                    to_bound,
                    context.clone().into(),
                    None,
                )
                .await?;
            let hourly_markers: Vec<ExtraHoursMarkerTO> = raw
                .iter()
                .filter(|eh| is_absence_category(&eh.category))
                .map(|eh| map_to_marker(eh, person.name.clone()))
                .collect();

            let result = AbsenceListWithProjectionTO {
                absence_periods,
                hourly_markers,
            };
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&result).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        create_absence_period,
        get_all_absence_periods,
        get_absence_period,
        update_absence_period,
        delete_absence_period,
        get_absence_periods_for_sales_person,
    ),
    components(schemas(
        AbsencePeriodTO,
        AbsenceCategoryTO,
        AbsencePeriodCreateResultTO,
        WarningTO,
        // Phase 8.5 (Plan 04): Read-Projektion Schemas.
        ExtraHoursMarkerTO,
        AbsenceListWithProjectionTO,
    )),
    tags(
        (name = "Absence", description = "Absence period management (range-based)"),
    ),
)]
pub struct AbsenceApiDoc;
