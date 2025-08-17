use axum::body::Body;
use axum::extract::Path;
use axum::routing::post;
use axum::{extract::State, response::Response};
use axum::{Extension, Router};
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use service::block_report::BlockReportService;
use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/{template_id}", post(generate_block_report::<RestState>))
}

#[utoipa::path(
    post,
    path = "/{template_id}",
    params(
        ("template_id" = Uuid, Path, description = "Text template ID")
    ),
    responses(
        (status = 200, description = "Block report generated successfully", body = String, content_type = "text/plain"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - HR permission required"),
        (status = 404, description = "Template not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
#[instrument(skip(rest_state))]
pub async fn generate_block_report<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(template_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let block_report = rest_state
                .block_report_service()
                .generate_block_report(template_id, context.into(), None)
                .await?;

            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(Body::new(block_report.to_string()))
                .unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        generate_block_report,
    ),
    tags(
        (name = "block_report", description = "Block Report management")
    )
)]
pub struct BlockReportApiDoc;