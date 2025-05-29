use std::rc::Rc;

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use rest_types::WeekMessageTO;
use serde::Deserialize;
use service::week_message::WeekMessageService;
use tracing::instrument;
use utoipa::{IntoParams, OpenApi};
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", post(create_week_message::<RestState>))
        .route("/{id}", get(get_week_message_by_id::<RestState>))
        .route("/{id}", put(update_week_message::<RestState>))
        .route("/{id}", delete(delete_week_message::<RestState>))
        .route(
            "/by-year/{year}",
            get(get_week_messages_by_year::<RestState>),
        )
        .route(
            "/by-year-and-week/{year}/{week}",
            get(get_week_message_by_year_and_week::<RestState>),
        )
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct WeekMessageQueryParams {
    #[param(example = "2025")]
    pub year: Option<u32>,

    #[param(example = "3")]
    #[serde(rename = "calendar-week")]
    pub calendar_week: Option<u8>,
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["Week Messages"],
    request_body = WeekMessageTO,
    responses(
        (status = 201, description = "Week message created", body = WeekMessageTO),
        (status = 400, description = "Invalid input"),
        (status = 403, description = "Forbidden"),
    ),
)]
pub async fn create_week_message<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(week_message): Json<WeekMessageTO>,
) -> Response {
    error_handler(
        (async {
            let week_message = WeekMessageTO::from(
                &rest_state
                    .week_message_service()
                    .create(&(&week_message).into(), context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&week_message).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}",
    tags = ["Week Messages"],
    params(
        ("id", description = "Week message id", example = "1a2b3c4d-5e6f-7g8h-9i0j-k1l2m3n4o5p6"),
    ),
    responses(
        (status = 200, description = "Week message found", body = WeekMessageTO),
        (status = 404, description = "Week message not found"),
    ),
)]
pub async fn get_week_message_by_id<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let week_message = rest_state
                .week_message_service()
                .get_by_id(id, context.into(), None)
                .await?;

            match week_message {
                Some(message) => {
                    let week_message_to = WeekMessageTO::from(&message);
                    Ok(Response::builder()
                        .status(200)
                        .header("Content-Type", "application/json")
                        .body(Body::new(serde_json::to_string(&week_message_to).unwrap()))
                        .unwrap())
                }
                None => Ok(Response::builder().status(404).body(Body::empty()).unwrap()),
            }
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/{id}",
    tags = ["Week Messages"],
    params(
        ("id", description = "Week message id", example = "1a2b3c4d-5e6f-7g8h-9i0j-k1l2m3n4o5p6"),
    ),
    request_body = WeekMessageTO,
    responses(
        (status = 200, description = "Week message updated", body = WeekMessageTO),
        (status = 400, description = "Invalid input"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Week message not found"),
    ),
)]
pub async fn update_week_message<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
    Json(mut week_message): Json<WeekMessageTO>,
) -> Response {
    error_handler(
        (async {
            week_message.id = id; // Ensure the ID matches the path parameter
            let week_message = WeekMessageTO::from(
                &rest_state
                    .week_message_service()
                    .update(&(&week_message).into(), context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&week_message).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/{id}",
    tags = ["Week Messages"],
    params(
        ("id", description = "Week message id", example = "1a2b3c4d-5e6f-7g8h-9i0j-k1l2m3n4o5p6"),
    ),
    responses(
        (status = 204, description = "Week message deleted"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Week message not found"),
    ),
)]
pub async fn delete_week_message<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .week_message_service()
                .delete(id, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/by-year/{year}",
    tags = ["Week Messages"],
    params(
        ("year", description = "Year", example = "2025"),
    ),
    responses(
        (status = 200, description = "Week messages for year", body = [WeekMessageTO]),
    ),
)]
pub async fn get_week_messages_by_year<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(year): Path<u32>,
) -> Response {
    error_handler(
        (async {
            let week_messages: Rc<[WeekMessageTO]> = rest_state
                .week_message_service()
                .get_by_year(year, context.into(), None)
                .await?
                .iter()
                .map(WeekMessageTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&week_messages).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/by-year-and-week/{year}/{week}",
    tags = ["Week Messages"],
    params(
        ("year", description = "Year", example = "2025"),
        ("week", description = "Calendar week", example = "20"),
    ),
    responses(
        (status = 200, description = "Week message found", body = WeekMessageTO),
        (status = 404, description = "Week message not found"),
    ),
)]
pub async fn get_week_message_by_year_and_week<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((year, week)): Path<(u32, u8)>,
) -> Response {
    error_handler(
        (async {
            let week_message = rest_state
                .week_message_service()
                .get_by_year_and_week(year, week, context.into(), None)
                .await?;

            match week_message {
                Some(message) => {
                    let week_message_to = WeekMessageTO::from(&message);
                    Ok(Response::builder()
                        .status(200)
                        .header("Content-Type", "application/json")
                        .body(Body::new(serde_json::to_string(&week_message_to).unwrap()))
                        .unwrap())
                }
                None => Ok(Response::builder().status(404).body(Body::empty()).unwrap()),
            }
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        create_week_message,
        get_week_message_by_id,
        update_week_message,
        delete_week_message,
        get_week_messages_by_year,
        get_week_message_by_year_and_week,
    ),
    components(schemas(WeekMessageTO))
)]
pub struct WeekMessageApiDoc;
