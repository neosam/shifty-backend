use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use rest_types::{ToggleGroupTO, ToggleTO};
use service::toggle::ToggleService;
use tracing::instrument;
use utoipa::OpenApi;

use crate::{error_handler, Context, RestStateDef};

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        // Toggle endpoints
        .route("/", get(get_all_toggles::<RestState>))
        .route("/", post(create_toggle::<RestState>))
        .route("/{name}", get(get_toggle::<RestState>))
        .route("/{name}/enabled", get(is_toggle_enabled::<RestState>))
        .route("/{name}/enable", put(enable_toggle::<RestState>))
        .route("/{name}/disable", put(disable_toggle::<RestState>))
        .route("/{name}", delete(delete_toggle::<RestState>))
}

pub fn generate_group_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        // Toggle group endpoints
        .route("/", get(get_all_toggle_groups::<RestState>))
        .route("/", post(create_toggle_group::<RestState>))
        .route("/{name}", get(get_toggle_group::<RestState>))
        .route("/{name}", delete(delete_toggle_group::<RestState>))
        .route(
            "/{group}/toggle/{toggle}",
            post(add_toggle_to_group::<RestState>),
        )
        .route(
            "/{group}/toggle/{toggle}",
            delete(remove_toggle_from_group::<RestState>),
        )
        .route("/{name}/toggles", get(get_toggles_in_group::<RestState>))
        .route("/{name}/enable", put(enable_group::<RestState>))
        .route("/{name}/disable", put(disable_group::<RestState>))
}

// Toggle endpoints

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "",
    tags = ["Toggles"],
    responses(
        (status = 200, description = "List of all toggles", body = [ToggleTO]),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn get_all_toggles<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let toggles: Arc<[ToggleTO]> = rest_state
                .toggle_service()
                .get_all_toggles(context.into(), None)
                .await?
                .iter()
                .map(ToggleTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&toggles).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{name}",
    tags = ["Toggles"],
    params(
        ("name", description = "Toggle name", example = "dark_mode"),
    ),
    responses(
        (status = 200, description = "Toggle found", body = ToggleTO),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Toggle not found"),
    ),
)]
pub async fn get_toggle<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(name): Path<String>,
) -> Response {
    error_handler(
        (async {
            let toggle = rest_state
                .toggle_service()
                .get_toggle(&name, context.into(), None)
                .await?;

            match toggle {
                Some(t) => {
                    let toggle_to = ToggleTO::from(&t);
                    Ok(Response::builder()
                        .status(200)
                        .header("Content-Type", "application/json")
                        .body(Body::new(serde_json::to_string(&toggle_to).unwrap()))
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
    get,
    path = "/{name}/enabled",
    tags = ["Toggles"],
    params(
        ("name", description = "Toggle name", example = "dark_mode"),
    ),
    responses(
        (status = 200, description = "Toggle enabled status", body = bool),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn is_toggle_enabled<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(name): Path<String>,
) -> Response {
    error_handler(
        (async {
            let enabled = rest_state
                .toggle_service()
                .is_enabled(&name, context.into(), None)
                .await?;
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&enabled).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["Toggles"],
    request_body = ToggleTO,
    responses(
        (status = 201, description = "Toggle created"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
    ),
)]
pub async fn create_toggle<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(toggle): Json<ToggleTO>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .toggle_service()
                .create_toggle(&(&toggle).into(), context.into(), None)
                .await?;
            Ok(Response::builder().status(201).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/{name}/enable",
    tags = ["Toggles"],
    params(
        ("name", description = "Toggle name", example = "dark_mode"),
    ),
    responses(
        (status = 204, description = "Toggle enabled"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
        (status = 404, description = "Toggle not found"),
    ),
)]
pub async fn enable_toggle<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(name): Path<String>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .toggle_service()
                .enable_toggle(&name, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/{name}/disable",
    tags = ["Toggles"],
    params(
        ("name", description = "Toggle name", example = "dark_mode"),
    ),
    responses(
        (status = 204, description = "Toggle disabled"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
        (status = 404, description = "Toggle not found"),
    ),
)]
pub async fn disable_toggle<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(name): Path<String>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .toggle_service()
                .disable_toggle(&name, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/{name}",
    tags = ["Toggles"],
    params(
        ("name", description = "Toggle name", example = "dark_mode"),
    ),
    responses(
        (status = 204, description = "Toggle deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
    ),
)]
pub async fn delete_toggle<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(name): Path<String>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .toggle_service()
                .delete_toggle(&name, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

// Toggle group endpoints

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "",
    tags = ["Toggle Groups"],
    responses(
        (status = 200, description = "List of all toggle groups", body = [ToggleGroupTO]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
    ),
)]
pub async fn get_all_toggle_groups<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let groups: Arc<[ToggleGroupTO]> = rest_state
                .toggle_service()
                .get_all_toggle_groups(context.into(), None)
                .await?
                .iter()
                .map(ToggleGroupTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&groups).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{name}",
    tags = ["Toggle Groups"],
    params(
        ("name", description = "Toggle group name", example = "experimental"),
    ),
    responses(
        (status = 200, description = "Toggle group found", body = ToggleGroupTO),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
        (status = 404, description = "Toggle group not found"),
    ),
)]
pub async fn get_toggle_group<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(name): Path<String>,
) -> Response {
    error_handler(
        (async {
            let group = rest_state
                .toggle_service()
                .get_toggle_group(&name, context.into(), None)
                .await?;

            match group {
                Some(g) => {
                    let group_to = ToggleGroupTO::from(&g);
                    Ok(Response::builder()
                        .status(200)
                        .header("Content-Type", "application/json")
                        .body(Body::new(serde_json::to_string(&group_to).unwrap()))
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
    post,
    path = "",
    tags = ["Toggle Groups"],
    request_body = ToggleGroupTO,
    responses(
        (status = 201, description = "Toggle group created"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
    ),
)]
pub async fn create_toggle_group<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(group): Json<ToggleGroupTO>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .toggle_service()
                .create_toggle_group(&(&group).into(), context.into(), None)
                .await?;
            Ok(Response::builder().status(201).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/{name}",
    tags = ["Toggle Groups"],
    params(
        ("name", description = "Toggle group name", example = "experimental"),
    ),
    responses(
        (status = 204, description = "Toggle group deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
    ),
)]
pub async fn delete_toggle_group<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(name): Path<String>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .toggle_service()
                .delete_toggle_group(&name, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "/{group}/toggle/{toggle}",
    tags = ["Toggle Groups"],
    params(
        ("group", description = "Toggle group name", example = "experimental"),
        ("toggle", description = "Toggle name", example = "dark_mode"),
    ),
    responses(
        (status = 201, description = "Toggle added to group"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
    ),
)]
pub async fn add_toggle_to_group<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((group, toggle)): Path<(String, String)>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .toggle_service()
                .add_toggle_to_group(&group, &toggle, context.into(), None)
                .await?;
            Ok(Response::builder().status(201).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    delete,
    path = "/{group}/toggle/{toggle}",
    tags = ["Toggle Groups"],
    params(
        ("group", description = "Toggle group name", example = "experimental"),
        ("toggle", description = "Toggle name", example = "dark_mode"),
    ),
    responses(
        (status = 204, description = "Toggle removed from group"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
    ),
)]
pub async fn remove_toggle_from_group<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path((group, toggle)): Path<(String, String)>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .toggle_service()
                .remove_toggle_from_group(&group, &toggle, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{name}/toggles",
    tags = ["Toggle Groups"],
    params(
        ("name", description = "Toggle group name", example = "experimental"),
    ),
    responses(
        (status = 200, description = "List of toggles in group", body = [ToggleTO]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
    ),
)]
pub async fn get_toggles_in_group<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(name): Path<String>,
) -> Response {
    error_handler(
        (async {
            let toggles: Arc<[ToggleTO]> = rest_state
                .toggle_service()
                .get_toggles_in_group(&name, context.into(), None)
                .await?
                .iter()
                .map(ToggleTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&toggles).unwrap()))
                .unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/{name}/enable",
    tags = ["Toggle Groups"],
    params(
        ("name", description = "Toggle group name", example = "experimental"),
    ),
    responses(
        (status = 204, description = "All toggles in group enabled"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
    ),
)]
pub async fn enable_group<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(name): Path<String>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .toggle_service()
                .enable_group(&name, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    put,
    path = "/{name}/disable",
    tags = ["Toggle Groups"],
    params(
        ("name", description = "Toggle group name", example = "experimental"),
    ),
    responses(
        (status = 204, description = "All toggles in group disabled"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires toggle_admin privilege"),
    ),
)]
pub async fn disable_group<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(name): Path<String>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .toggle_service()
                .disable_group(&name, context.into(), None)
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_toggles,
        get_toggle,
        is_toggle_enabled,
        create_toggle,
        enable_toggle,
        disable_toggle,
        delete_toggle,
    ),
    components(schemas(ToggleTO))
)]
pub struct ToggleApiDoc;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_toggle_groups,
        get_toggle_group,
        create_toggle_group,
        delete_toggle_group,
        add_toggle_to_group,
        remove_toggle_from_group,
        get_toggles_in_group,
        enable_group,
        disable_group,
    ),
    components(schemas(ToggleGroupTO))
)]
pub struct ToggleGroupApiDoc;
