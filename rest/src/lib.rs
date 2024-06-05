use std::{convert::Infallible, sync::Arc};

mod booking;
mod permission;
mod sales_person;
mod slot;

use axum::extract::Request;
use axum::http::Uri;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{body::Body, error_handling::HandleErrorLayer, response::Response, Router};
#[cfg(feature = "oidc")]
use axum_oidc::{EmptyAdditionalClaims, OidcClaims};
use service::ServiceError;
use thiserror::Error;
use time::Duration;
use tower::ServiceBuilder;
use tower_sessions::{cookie::SameSite, Expiry, MemoryStore, SessionManagerLayer};
use uuid::Uuid;

// TODO: In prod, it must be a different type than in dev mode.
#[cfg(feature = "mock_auth")]
type Context = ();
#[cfg(feature = "oidc")]
type Context = Option<Arc<str>>;

#[cfg(feature = "oidc")]
pub async fn context_extractor(
    claims: Option<OidcClaims<EmptyAdditionalClaims>>,
    mut request: Request,
    next: Next,
) -> Response {
    let context: Context = if let Some(oidc_claims) = claims {
        let username = oidc_claims
            .preferred_username()
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|| "NoUsername".to_string());
        Some(username.into())
    } else {
        None
    };
    request.extensions_mut().insert(context);
    next.run(request).await
}
#[cfg(feature = "mock_auth")]
pub async fn context_extractor(mut request: Request, next: Next) -> Response {
    request.extensions_mut().insert(());
    next.run(request).await
}

pub struct RoString(Arc<str>, bool);
impl http_body::Body for RoString {
    type Data = bytes::Bytes;
    type Error = Infallible;

    fn poll_frame(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        std::task::Poll::Ready(if self.1 {
            None
        } else {
            self.1 = true;
            Some(Ok(http_body::Frame::data(bytes::Bytes::copy_from_slice(
                self.0.as_bytes(),
            ))))
        })
    }

    fn is_end_stream(&self) -> bool {
        self.1
    }
}
impl From<Arc<str>> for RoString {
    fn from(s: Arc<str>) -> Self {
        RoString(s, false)
    }
}
impl From<RoString> for Response {
    fn from(s: RoString) -> Self {
        Response::builder().status(200).body(Body::new(s)).unwrap()
    }
}

#[derive(Debug, Error)]
pub enum RestError {
    #[error("Service error")]
    ServiceError(#[from] service::ServiceError),

    #[error("Inconsistent id. Got {0} in path but {1} in body")]
    InconsistentId(Uuid, Uuid),
}

fn error_handler(result: Result<Response, RestError>) -> Response {
    if result.is_err() {
        println!("REST error mapping: {:?}", result);
    }
    match result {
        Ok(response) => response,
        Err(err @ RestError::InconsistentId(_, _)) => Response::builder()
            .status(400)
            .body(Body::new(err.to_string()))
            .unwrap(),
        Err(RestError::ServiceError(service::ServiceError::Forbidden)) => {
            Response::builder().status(403).body(Body::empty()).unwrap()
        }
        Err(RestError::ServiceError(service::ServiceError::Unauthorized)) => {
            Response::builder().status(401).body(Body::empty()).unwrap()
        }
        Err(RestError::ServiceError(service::ServiceError::DatabaseQueryError(e))) => {
            Response::builder()
                .status(500)
                .body(Body::new(e.to_string()))
                .unwrap()
        }
        Err(RestError::ServiceError(service::ServiceError::EntityAlreadyExists(id))) => {
            Response::builder()
                .status(409)
                .body(Body::new(id.to_string()))
                .unwrap()
        }
        Err(RestError::ServiceError(service::ServiceError::EntityNotFound(id))) => {
            Response::builder()
                .status(404)
                .body(Body::new(id.to_string()))
                .unwrap()
        }
        Err(RestError::ServiceError(err @ service::ServiceError::EntityConflicts(_, _, _))) => {
            Response::builder()
                .status(409)
                .body(Body::new(err.to_string()))
                .unwrap()
        }
        Err(RestError::ServiceError(err @ service::ServiceError::ValidationError(_))) => {
            Response::builder()
                .status(422)
                .body(Body::new(err.to_string()))
                .unwrap()
        }
        Err(RestError::ServiceError(err @ service::ServiceError::IdSetOnCreate)) => {
            Response::builder()
                .status(422)
                .body(Body::new(err.to_string()))
                .unwrap()
        }
        Err(RestError::ServiceError(err @ service::ServiceError::VersionSetOnCreate)) => {
            Response::builder()
                .status(422)
                .body(Body::new(err.to_string()))
                .unwrap()
        }
        Err(RestError::ServiceError(err @ service::ServiceError::OverlappingTimeRange)) => {
            Response::builder()
                .status(409)
                .body(Body::new(err.to_string()))
                .unwrap()
        }
        Err(RestError::ServiceError(err @ service::ServiceError::TimeOrderWrong(_, _))) => {
            Response::builder()
                .status(422)
                .body(Body::new(err.to_string()))
                .unwrap()
        }
        Err(RestError::ServiceError(err @ service::ServiceError::DateOrderWrong(_, _))) => {
            Response::builder()
                .status(422)
                .body(Body::new(err.to_string()))
                .unwrap()
        }
        Err(RestError::ServiceError(ServiceError::InternalError)) => Response::builder()
            .status(500)
            .body(Body::new("Internal server error".to_string()))
            .unwrap(),
    }
}

pub trait RestStateDef: Clone + Send + Sync + 'static {
    type PermissionService: service::PermissionService<Context = Context> + Send + Sync + 'static;
    type SlotService: service::slot::SlotService<Context = Context> + Send + Sync + 'static;
    type SalesPersonService: service::sales_person::SalesPersonService<Context = Context>
        + Send
        + Sync
        + 'static;
    type BookingService: service::booking::BookingService<Context = Context> + Send + Sync + 'static;

    fn permission_service(&self) -> Arc<Self::PermissionService>;
    fn slot_service(&self) -> Arc<Self::SlotService>;
    fn sales_person_service(&self) -> Arc<Self::SalesPersonService>;
    fn booking_service(&self) -> Arc<Self::BookingService>;
}

pub struct OidcConfig {
    pub app_url: String,
    pub issuer: String,
    pub client_id: String,
    pub client_secret: Option<String>,
}
pub fn oidc_config() -> OidcConfig {
    let app_url = std::env::var("APP_URL").expect("APP_URL env variable");
    let issuer = std::env::var("ISSUER").expect("ISSUER env variable");
    let client_id = std::env::var("CLIENT_ID").expect("CLIENT_ID env variable");
    let client_secret = std::env::var("CLIENT_SECRET").ok();
    OidcConfig {
        app_url: app_url.into(),
        issuer: issuer.into(),
        client_id: client_id.into(),
        client_secret: client_secret.unwrap_or_default().into(),
    }
}

pub fn bind_address() -> Arc<str> {
    std::env::var("SERVER_ADDRESS")
        .unwrap_or("127.0.0.1:3000".into())
        .into()
}

pub async fn login() -> Redirect {
    Redirect::to("/")
}

#[cfg(feature = "oidc")]
pub async fn auth_info(claims: Option<OidcClaims<EmptyAdditionalClaims>>) -> Response {
    if let Some(oidc_claims) = claims {
        let username = oidc_claims
            .preferred_username()
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|| "NoUsername".to_string());
        let email = oidc_claims
            .email()
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|| "MailNotSet".to_string());
        let name = oidc_claims
            .name()
            .map(|s| {
                s.iter()
                    .next()
                    .map(|s| s.1.as_str().to_string())
                    .unwrap_or_else(|| "NoLocalizedName".to_string())
            })
            .unwrap_or_else(|| "NameNotSet".to_string());
        let body = format!(
            "Hello, {}! Your email is {} and your username is {}",
            name, email, username
        );
        Response::builder()
            .status(200)
            .body(Body::new(body))
            .unwrap()
    } else {
        Response::builder().status(401).body(Body::empty()).unwrap()
    }
}

pub async fn start_server<RestState: RestStateDef>(rest_state: RestState) {
    let app = Router::new();

    #[cfg(feature = "oidc")]
    let app = {
        use axum_oidc::error::MiddlewareError;
        use axum_oidc::{EmptyAdditionalClaims, OidcAuthLayer, OidcLoginLayer};

        let oidc_login_service = ServiceBuilder::new()
            .layer(HandleErrorLayer::new(|e: MiddlewareError| async {
                e.into_response()
            }))
            .layer(OidcLoginLayer::<EmptyAdditionalClaims>::new());

        app.route("/authenticate", get(login))
            .layer(oidc_login_service)
            .route("/auth-info", get(auth_info))
    };

    let app = app
        .nest("/permission", permission::generate_route())
        .nest("/slot", slot::generate_route())
        .nest("/sales-person", sales_person::generate_route())
        .nest("/booking", booking::generate_route())
        .with_state(rest_state)
        .layer(middleware::from_fn(context_extractor));

    #[cfg(feature = "oidc")]
    let app = {
        use axum_oidc::error::MiddlewareError;
        use axum_oidc::{EmptyAdditionalClaims, OidcAuthLayer, OidcLoginLayer};

        let oidc_config = oidc_config();
        let session_store = MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(true)
            .with_same_site(SameSite::Strict)
            .with_expiry(Expiry::OnSessionEnd);

        let oidc_auth_service = ServiceBuilder::new()
            .layer(HandleErrorLayer::new(|e: MiddlewareError| async {
                e.into_response()
            }))
            .layer(
                OidcAuthLayer::<EmptyAdditionalClaims>::discover_client(
                    Uri::from_maybe_shared(oidc_config.app_url).expect("valid APP_URL"),
                    oidc_config.issuer,
                    oidc_config.client_id,
                    oidc_config.client_secret,
                    vec![],
                )
                .await
                .unwrap(),
            );

        app.layer(oidc_auth_service).layer(session_layer)
    };

    let listener = tokio::net::TcpListener::bind(bind_address().as_ref())
        .await
        .expect("Could not bind server");
    axum::serve(listener, app)
        .await
        .expect("Could not start server");
}
