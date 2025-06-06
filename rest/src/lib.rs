use std::{convert::Infallible, sync::Arc};

mod booking;
mod booking_information;
mod custom_extra_hours;
mod employee_work_details;
mod extra_hours;
mod permission;
mod report;
mod sales_person;
mod session;
mod shiftplan;
mod shiftplan_edit;
mod slot;
mod special_day;
mod week_message;

#[cfg(feature = "oidc")]
use axum::error_handling::HandleErrorLayer;
use axum::extract::State;
#[cfg(feature = "oidc")]
use axum::http::Uri;
use axum::middleware::{self};
#[cfg(feature = "oidc")]
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum::routing::get;
use axum::Extension;
use axum::{body::Body, response::Response, Router};
use custom_extra_hours::CustomExtraHoursApiDoc;
use sales_person::SalesPersonApiDoc;
use serde::{Deserialize, Serialize};
use service::user_service::UserService;
use service::PermissionService;
use service::ServiceError;
pub use session::Context;
use session::{context_extractor, forbid_unauthenticated};
use thiserror::Error;
#[cfg(feature = "oidc")]
use time::Duration;
#[cfg(feature = "oidc")]
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
#[cfg(feature = "oidc")]
use tower_sessions::MemoryStore;
#[cfg(feature = "oidc")]
use tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer};
use tracing::info;
use utoipa::openapi::ServerBuilder;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

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

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Parse int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
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
        Err(err @ RestError::BadRequest(_)) => Response::builder()
            .status(400)
            .body(Body::new(err.to_string()))
            .unwrap(),
        Err(err @ RestError::ParseIntError(_)) => Response::builder()
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
        Err(RestError::ServiceError(service::ServiceError::EntityNotFoundGeneric(description))) => {
            Response::builder()
                .status(404)
                .body(Body::new(description.to_string()))
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
        Err(RestError::ServiceError(err @ service::ServiceError::DeletedSetOnCreate)) => {
            Response::builder()
                .status(422)
                .body(Body::new(err.to_string()))
                .unwrap()
        }
        Err(RestError::ServiceError(err @ service::ServiceError::CreatedSetOnCreate)) => {
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
        Err(RestError::ServiceError(err @ service::ServiceError::TimeComponentRangeError(_))) => {
            Response::builder()
                .status(500)
                .body(Body::new(err.to_string()))
                .unwrap()
        }
        Err(RestError::ServiceError(err @ service::ServiceError::TimeFormatError(_))) => {
            Response::builder()
                .status(500)
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
    type UserService: service::user_service::UserService<Context = Context> + Send + Sync + 'static;
    type SessionService: service::session::SessionService<Context = Context> + Send + Sync + 'static;
    type PermissionService: service::PermissionService<Context = Context> + Send + Sync + 'static;
    type SlotService: service::slot::SlotService<Context = Context> + Send + Sync + 'static;
    type SalesPersonService: service::sales_person::SalesPersonService<Context = Context>
        + Send
        + Sync
        + 'static;
    type SpecialDayService: service::special_days::SpecialDayService<Context = Context>
        + Send
        + Sync
        + 'static;
    type SalesPersonUnavailableService: service::sales_person_unavailable::SalesPersonUnavailableService<Context = Context>
        + Send
        + Sync
        + 'static;
    type BookingService: service::booking::BookingService<Context = Context> + Send + Sync + 'static;
    type CustomExtraHoursService: service::custom_extra_hours::CustomExtraHoursService<Context = Context>
        + Send
        + Sync
        + 'static;
    type BookingInformationService: service::booking_information::BookingInformationService<Context = Context>
        + Send
        + Sync
        + 'static;
    type ReportingService: service::reporting::ReportingService<Context = Context>
        + Send
        + Sync
        + 'static;
    type WorkingHoursService: service::employee_work_details::EmployeeWorkDetailsService<Context = Context>
        + Send
        + Sync
        + 'static;
    type ExtraHoursService: service::extra_hours::ExtraHoursService<Context = Context>
        + Send
        + Sync
        + 'static;
    type ShiftplanEditService: service::shiftplan_edit::ShiftplanEditService<Context = Context>
        + Send
        + Sync
        + 'static;
    type BlockService: service::block::BlockService<Context = Context> + Send + Sync + 'static;
    type ShiftplanService: service::shiftplan::ShiftplanService<Context = Context>
        + Send
        + Sync
        + 'static;
    type WeekMessageService: service::week_message::WeekMessageService<Context = Context>
        + Send
        + Sync
        + 'static;

    fn backend_version(&self) -> Arc<str>;

    fn user_service(&self) -> Arc<Self::UserService>;
    fn session_service(&self) -> Arc<Self::SessionService>;
    fn permission_service(&self) -> Arc<Self::PermissionService>;
    fn slot_service(&self) -> Arc<Self::SlotService>;
    fn sales_person_service(&self) -> Arc<Self::SalesPersonService>;
    fn special_day_service(&self) -> Arc<Self::SpecialDayService>;
    fn sales_person_unavailable_service(&self) -> Arc<Self::SalesPersonUnavailableService>;
    fn booking_service(&self) -> Arc<Self::BookingService>;
    fn custom_extra_hours_service(&self) -> Arc<Self::CustomExtraHoursService>;
    fn booking_information_service(&self) -> Arc<Self::BookingInformationService>;
    fn reporting_service(&self) -> Arc<Self::ReportingService>;
    fn working_hours_service(&self) -> Arc<Self::WorkingHoursService>;
    fn extra_hours_service(&self) -> Arc<Self::ExtraHoursService>;
    fn shiftplan_edit_service(&self) -> Arc<Self::ShiftplanEditService>;
    fn block_service(&self) -> Arc<Self::BlockService>;
    fn shiftplan_service(&self) -> Arc<Self::ShiftplanService>;
    fn week_message_service(&self) -> Arc<Self::WeekMessageService>;
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
        app_url,
        issuer,
        client_id,
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

pub async fn get_version<RestState: RestStateDef>(rest_state: State<RestState>) -> Response {
    Response::builder()
        .status(200)
        .body(Body::new(rest_state.backend_version().to_string()))
        .unwrap()
}

#[cfg(feature = "oidc")]
use axum_oidc::OidcRpInitiatedLogout;
#[cfg(feature = "oidc")]
use http::StatusCode;
#[cfg(feature = "oidc")]
pub async fn logout(logout_extractor: OidcRpInitiatedLogout) -> Result<Redirect, StatusCode> {
    if let Ok(logout_uri) = logout_extractor.uri() {
        Ok(Redirect::to(&format!("{}", logout_uri)))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthInfoTO {
    pub user: Arc<str>,
    pub privileges: Arc<[Arc<str>]>,
}

pub async fn auth_info<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let user = rest_state
                .user_service()
                .current_user(context.clone())
                .await?;
            let privileges: Arc<[Arc<str>]> = rest_state
                .permission_service()
                .get_privileges_for_current_user(context.into())
                .await?
                .iter()
                .map(|privilege| privilege.name.clone())
                .collect();
            let auth_info = AuthInfoTO { user, privileges };

            let response = serde_json::to_string(&auth_info).unwrap();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(response))
                .unwrap())
        })
        .await,
    )
}

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/custom-extra-hours", api = CustomExtraHoursApiDoc),
        (path = "/sales-person", api = SalesPersonApiDoc),
        (path = "/extra-hours", api = extra_hours::ExtraHoursApiDoc),
        (path = "/report", api = report::ReportApiDoc),
        (path = "/week-message", api = week_message::WeekMessageApiDoc),
        (path = "/permission", api = permission::PermissionApiDoc),
    )
)]
pub struct ApiDoc;

pub async fn start_server<RestState: RestStateDef>(rest_state: RestState) {
    let app = Router::new();

    let app = app.route("/authenticate", get(login));

    #[cfg(feature = "oidc")]
    let app = {
        use axum_oidc::error::MiddlewareError;
        use axum_oidc::{EmptyAdditionalClaims, OidcLoginLayer};

        let oidc_login_service = ServiceBuilder::new()
            .layer(HandleErrorLayer::new(|e: MiddlewareError| async {
                e.into_response()
            }))
            .layer(OidcLoginLayer::<EmptyAdditionalClaims>::new());

        app.route("/logout", get(logout))
            .layer(middleware::from_fn_with_state(
                rest_state.clone(),
                session::register_session::<RestState>,
            ))
            .layer(oidc_login_service)
    };

    let mut api_doc = ApiDoc::openapi();
    let base = std::env::var("BASE_PATH").unwrap_or("http://localhost:3000/".into());
    api_doc.servers = Some(vec![ServerBuilder::new()
        .url(base)
        .description(Some("Shifty backend"))
        .build()]);
    let swagger_router = SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api_doc);
    let app = app.merge(swagger_router);

    let app = app
        .route("/auth-info", get(auth_info::<RestState>))
        .route("/version", get(get_version::<RestState>))
        .nest("/permission", permission::generate_route())
        .nest("/slot", slot::generate_route())
        .nest("/sales-person", sales_person::generate_route())
        .nest("/booking", booking::generate_route())
        .nest("/custom-extra-hours", custom_extra_hours::generate_route())
        .nest(
            "/booking-information",
            booking_information::generate_route(),
        )
        .nest("/report", report::generate_route())
        .nest("/working-hours", employee_work_details::generate_route())
        .nest(
            "/employee-work-details",
            employee_work_details::generate_route(),
        )
        .nest("/extra-hours", extra_hours::generate_route())
        .nest("/special-days", special_day::generate_route())
        .nest("/shiftplan-edit", shiftplan_edit::generate_route())
        .nest("/shiftplan-info", shiftplan::generate_route())
        .nest("/week-message", week_message::generate_route())
        .with_state(rest_state.clone())
        .layer(middleware::from_fn_with_state(
            rest_state.clone(),
            forbid_unauthenticated::<RestState>,
        ))
        .layer(middleware::from_fn_with_state(
            rest_state,
            context_extractor::<RestState>,
        ))
        .layer(CookieManagerLayer::new());

    #[cfg(feature = "oidc")]
    let app = {
        use axum_oidc::error::MiddlewareError;
        use axum_oidc::{EmptyAdditionalClaims, OidcAuthLayer};

        let oidc_config = oidc_config();
        let session_store = MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(true)
            .with_same_site(SameSite::Strict)
            .with_expiry(Expiry::OnInactivity(Duration::minutes(50)));

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

    info!("Running server at {}", bind_address());

    let listener = tokio::net::TcpListener::bind(bind_address().as_ref())
        .await
        .expect("Could not bind server");
    axum::serve(listener, app)
        .await
        .expect("Could not start server");
}
