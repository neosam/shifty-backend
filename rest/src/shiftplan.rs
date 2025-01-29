use axum::{
    extract::{Path, State},
    routing::get,
    Extension, Router,
};

use crate::{error_handler, Context, RestStateDef, Response};
use service::permission::Authentication;

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/:year/:week", get(get_shiftplan_week::<RestState>))
}

async fn get_shiftplan_week<RestState: RestStateDef>(
    Path((year, week)): Path<(u32, u8)>,
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        async {
            let shiftplan = rest_state
                .shiftplan_service()
                .get_shiftplan_week(year, week, Authentication::Context(context), None)
                .await?;

            Ok(Response::builder()
                .status(200)
                .body(axum::body::Body::from(serde_json::to_string(&shiftplan).unwrap()))
                .unwrap())
        }
        .await,
    )
}
