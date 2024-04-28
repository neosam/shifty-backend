use axum::{extract::State, response::Response};

use crate::{error_handler, RestStateDef, RoString};
use service::HelloService;

pub async fn hello<RestState: RestStateDef>(State(rest_state): State<RestState>) -> Response {
    error_handler(
        (async {
            let string = rest_state.hello_service().hello().await?;
            Ok(RoString::from(string).into())
        })
        .await,
    )
}
