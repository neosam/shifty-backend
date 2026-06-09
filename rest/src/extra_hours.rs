use std::rc::Rc;

use axum::{
    body::Body,
    extract::{Path, Query, State},
    response::Response,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use rest_types::{AbsencePeriodTO, ConvertExtraHoursRequestTO, ExtraHoursTO};

use serde::Deserialize;
use service::{
    absence_conversion::AbsenceConversionService, extra_hours::ExtraHoursService,
};
use tracing::instrument;
use utoipa::{IntoParams, OpenApi};
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", post(create_extra_hours::<RestState>))
        .route("/{id}", put(update_extra_hours::<RestState>))
        .route("/{id}", delete(delete_extra_hours::<RestState>))
        .route(
            "/by-sales-person/{id}",
            get(get_extra_hours_for_sales_person::<RestState>),
        )
        .route(
            "/{id}/convert-to-absence",
            post(convert_extra_hours_to_absence::<RestState>),
        )
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ExtraHoursForSalesPersonAttributes {
    #[param(example = "2025")]
    year: u32,

    #[param(example = "20")]
    until_week: u8,
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/by-sales-person/{id}",

    tags = ["Extra Hours"],
    params(
        ("id", description = "Sales person id", example = "1a2b3c4d-5e6f-7g8h-9i0j-k1l2m3n4o5p6"),
        ExtraHoursForSalesPersonAttributes,
    ),
    responses(
        (status = 201, description = "Extra hours for sales person", body = [ExtraHoursTO]),
        (status = 404, description = "Sales person not found"),
    ),
)]
pub async fn get_extra_hours_for_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    query: Query<ExtraHoursForSalesPersonAttributes>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let extra_hours: Rc<[ExtraHoursTO]> = rest_state
                .extra_hours_service()
                .find_by_sales_person_id_and_year(
                    sales_person_id,
                    query.year,
                    query.until_week,
                    context.into(),
                    None,
                )
                .await?
                .iter()
                .map(ExtraHoursTO::from)
                .collect();
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&extra_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["Extra Hours"],
    request_body = ExtraHoursTO,
    responses(
        (status = 201, description = "Extra hours created", body = ExtraHoursTO),
        (status = 400, description = "Invalid input"),
    ),
)]
pub async fn create_extra_hours<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(sales_person): Json<ExtraHoursTO>,
) -> Response {
    error_handler(
        (async {
            let extra_hours = ExtraHoursTO::from(
                &rest_state
                    .extra_hours_service()
                    .create(&(&sales_person).into(), context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&extra_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/{id}",
    tags = ["Extra Hours"],
    params(
        ("id", description = "Extra hours id (logical id)", example = "1a2b3c4d-5e6f-7g8h-9i0j-k1l2m3n4o5p6"),
    ),
    request_body = ExtraHoursTO,
    responses(
        (status = 200, description = "Updated extra hours", body = ExtraHoursTO),
        (status = 400, description = "Invalid input"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Extra hours not found"),
        (status = 409, description = "Version conflict"),
    ),
)]
pub async fn update_extra_hours<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(extra_hours_id): Path<Uuid>,
    Json(extra_hours_to): Json<ExtraHoursTO>,
) -> Response {
    error_handler(
        (async {
            let mut entity: service::extra_hours::ExtraHours = (&extra_hours_to).into();
            entity.id = extra_hours_id;
            let updated = ExtraHoursTO::from(
                &rest_state
                    .extra_hours_service()
                    .update(&entity, context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&updated).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/{id}",
    tags = ["Extra Hours"],
    params(
        ("id", description = "Extra hours id", example = "1a2b3c4d-5e6f-7g8h-9i0j-k1l2m3n4o5p6"),
    ),
    responses(
        (status = 204, description = "Extra hours deleted"),
        (status = 404, description = "Extra hours not found"),
    ),
)]
pub async fn delete_extra_hours<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(extra_hours_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .extra_hours_service()
                .delete(extra_hours_id, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/{id}/convert-to-absence",
    tags = ["Extra Hours"],
    params(
        ("id", description = "Extra hours logical id"),
    ),
    request_body = ConvertExtraHoursRequestTO,
    responses(
        (status = 200, description = "Converted absence period", body = AbsencePeriodTO),
        (status = 403, description = "Forbidden — requires hr privilege"),
        (status = 404, description = "Extra hours not found or already soft-deleted"),
        (status = 422, description = "Validation error (DateOrderWrong or OverlappingPeriod)"),
    ),
)]
pub async fn convert_extra_hours_to_absence<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(extra_hours_id): Path<Uuid>,
    Json(body): Json<ConvertExtraHoursRequestTO>,
) -> Response {
    error_handler(
        (async {
            // DayFractionTO → service::absence::DayFraction via cfg(feature = "service-impl") From impl
            let day_fraction = body
                .day_fraction
                .as_ref()
                .map(|f| service::absence::DayFraction::from(f));
            let result = rest_state
                .absence_conversion_service()
                .convert_extra_hours_to_absence(
                    extra_hours_id,
                    body.start,
                    body.end,
                    day_fraction,
                    context.into(),
                    None,
                )
                .await?;
            let to = AbsencePeriodTO::from(&result);
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
        get_extra_hours_for_sales_person,
        create_extra_hours,
        update_extra_hours,
        delete_extra_hours,
        convert_extra_hours_to_absence
    ),
    components(schemas(ExtraHoursTO, ConvertExtraHoursRequestTO)),
    tags(
        (name = "Extra Hours", description = "Extra hours management"),
    ),
)]
pub struct ExtraHoursApiDoc;
