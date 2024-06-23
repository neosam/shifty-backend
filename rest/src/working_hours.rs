use axum::{
    body::Body, extract::State, response::Response, routing::post, Extension, Json, Router,
};
use rest_types::WorkingHoursTO;

use service::working_hours::WorkingHoursService;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/", post(create_working_hours::<RestState>))
}

pub async fn create_working_hours<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(working_hours): Json<WorkingHoursTO>,
) -> Response {
    error_handler(
        (async {
            let working_hours = WorkingHoursTO::from(
                &rest_state
                    .working_hours_service()
                    .create(&(&working_hours).into(), context.into())
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&working_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}
