use axum::{
    body::Body,
    extract::{Path, State},
    routing::get,
    Extension, Router,
};
use rest_types::BlockTO;
use service::block::BlockService;
use service::permission::Authentication;
use shifty_utils::ShiftyWeek;
use tracing::instrument;
use utoipa::OpenApi;

use crate::{error_handler, Context, Response, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route(
        "/{from_year}/{from_week}/{until_year}/{until_week}",
        get(get_blocks_for_current_user::<RestState>),
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{from_year}/{from_week}/{until_year}/{until_week}",
    params(
        ("from_year" = u32, Path, description = "Start year"),
        ("from_week" = u8, Path, description = "Start calendar week (1-53)"),
        ("until_year" = u32, Path, description = "End year"),
        ("until_week" = u8, Path, description = "End calendar week (1-53)")
    ),
    responses(
        (status = 200, description = "List of blocks for the current user within the specified week range", body = Vec<BlockTO>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error")
    ),
    tag = "blocks"
)]
async fn get_blocks_for_current_user<RestState: RestStateDef>(
    Path((from_year, from_week, until_year, until_week)): Path<(u32, u8, u32, u8)>,
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        async {
            let from = ShiftyWeek::new(from_year, from_week);
            let until = ShiftyWeek::new(until_year, until_week);

            let blocks = rest_state
                .block_service()
                .get_blocks_for_current_user(from, until, Authentication::Context(context), None)
                .await?;

            let blocks_to: Vec<BlockTO> = blocks.iter().map(BlockTO::from).collect();

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&blocks_to).unwrap()))
                .unwrap())
        }
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_blocks_for_current_user,
    ),
    components(
        schemas(
            BlockTO,
        )
    ),
    tags(
        (name = "blocks", description = "Retrieve blocks for the currently logged-in user")
    )
)]
pub struct MyBlockApiDoc;
