use axum::{
    body::Body, extract::State, response::Response, routing::post, Extension, Json, Router,
};
use rest_types::ExtraHoursTO;

use service::extra_hours::ExtraHoursService;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/", post(create_extra_hours::<RestState>))
}

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
                    .create(&(&sales_person).into(), context.into())
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&extra_hours).unwrap()))
                .unwrap())
        })
        .await,
    )
}
