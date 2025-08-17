use std::sync::Arc;

use axum::body::Body;
use axum::extract::Path;
use axum::routing::{delete, get, post, put};
use axum::{extract::State, response::Response};
use axum::{Extension, Json, Router};
use rest_types::{CreateTextTemplateRequestTO, TextTemplateTO, UpdateTextTemplateRequestTO};
use service::text_template::TextTemplateService;
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_all_text_templates::<RestState>))
        .route("/{id}", get(get_text_template::<RestState>))
        .route("/", post(create_text_template::<RestState>))
        .route("/{id}", put(update_text_template::<RestState>))
        .route("/{id}", delete(delete_text_template::<RestState>))
        .route("/by-type/{template_type}", get(get_text_templates_by_type::<RestState>))
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    tags = ["Text Templates"],
    path = "",
    responses(
        (status = 200, description = "Get all text templates", body = [TextTemplateTO]),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_all_text_templates<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let text_templates: Arc<[TextTemplateTO]> = rest_state
                .text_template_service()
                .get_all(context.into(), None)
                .await?
                .iter()
                .map(TextTemplateTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&text_templates).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}",
    tags = ["Text Templates"],
    description = "Get text template by ID",
    params(
        ("id", description = "Text template ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    responses(
        (status = 200, description = "Get text template by ID", body = TextTemplateTO),
        (status = 404, description = "Text template not found"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_text_template<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(text_template_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let text_template = TextTemplateTO::from(
                &rest_state
                    .text_template_service()
                    .get_by_id(text_template_id, context.into(), None)
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&text_template).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/by-type/{template_type}",
    tags = ["Text Templates"],
    description = "Get text templates by template type",
    params(
        ("template_type", description = "Template type", example = "email"),
    ),
    responses(
        (status = 200, description = "Get text templates by type", body = [TextTemplateTO]),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_text_templates_by_type<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(template_type): Path<String>,
) -> Response {
    error_handler(
        (async {
            let text_templates: Arc<[TextTemplateTO]> = rest_state
                .text_template_service()
                .get_by_template_type(&template_type, context.into(), None)
                .await?
                .iter()
                .map(TextTemplateTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&text_templates).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["Text Templates"],
    description = "Create a new text template",
    request_body = CreateTextTemplateRequestTO,
    responses(
        (status = 201, description = "Create text template", body = TextTemplateTO),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - HR permission required"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_text_template<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(payload): Json<CreateTextTemplateRequestTO>,
) -> Response {
    error_handler(
        (async {
            let text_template = service::text_template::TextTemplate {
                id: Uuid::new_v4(), // Will be overwritten in service
                name: payload.name,
                template_type: payload.template_type,
                template_text: payload.template_text,
                created_at: None,
                created_by: None,
                deleted: None,
                deleted_by: None,
                version: Uuid::new_v4(),
            };

            let created_template = rest_state
                .text_template_service()
                .create(&text_template, context.into(), None)
                .await?;

            let text_template_to = TextTemplateTO::from(&created_template);
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&text_template_to).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/{id}",
    tags = ["Text Templates"],
    description = "Update a text template",
    params(
        ("id", description = "Text template ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    request_body = UpdateTextTemplateRequestTO,
    responses(
        (status = 200, description = "Update text template", body = TextTemplateTO),
        (status = 404, description = "Text template not found"),
        (status = 400, description = "Inconsistent ID"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - HR permission required"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_text_template<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(text_template_id): Path<Uuid>,
    Json(payload): Json<UpdateTextTemplateRequestTO>,
) -> Response {
    error_handler(
        (async {
            // Get the existing template to preserve other fields
            let existing_template = rest_state
                .text_template_service()
                .get_by_id(text_template_id, context.clone().into(), None)
                .await?;

            let updated_template = service::text_template::TextTemplate {
                id: text_template_id,
                name: payload.name,
                template_type: payload.template_type,
                template_text: payload.template_text,
                created_at: existing_template.created_at,
                created_by: existing_template.created_by,
                deleted: existing_template.deleted,
                deleted_by: existing_template.deleted_by,
                version: existing_template.version, // Will be updated in service
            };

            let result_template = rest_state
                .text_template_service()
                .update(&updated_template, context.into(), None)
                .await?;

            let text_template_to = TextTemplateTO::from(&result_template);
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&text_template_to).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/{id}",
    tags = ["Text Templates"],
    description = "Delete a text template",
    params(
        ("id", description = "Text template ID", example = "123e4567-e89b-12d3-a456-426614174000"),
    ),
    responses(
        (status = 204, description = "Delete text template"),
        (status = 404, description = "Text template not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - HR permission required"),
        (status = 500, description = "Internal server error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_text_template<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(text_template_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .text_template_service()
                .delete(text_template_id, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Text Templates", description = "Text template management API"),
    ),
    paths(
        get_all_text_templates,
        get_text_template,
        get_text_templates_by_type,
        create_text_template,
        update_text_template,
        delete_text_template,
    ),
    components(
        schemas(
            TextTemplateTO,
            CreateTextTemplateRequestTO,
            UpdateTextTemplateRequestTO,
        ),
    ),
)]
pub struct TextTemplateApiDoc;