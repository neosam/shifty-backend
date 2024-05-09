use std::{convert::Infallible, sync::Arc};

mod booking;
mod permission;
mod sales_person;
mod slot;

use axum::{body::Body, response::Response, Router};
use service::ServiceError;
use thiserror::Error;
use uuid::Uuid;

// TODO: In prod, it must be a different type than in dev mode.
type Context = ();

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

pub async fn start_server<RestState: RestStateDef>(rest_state: RestState) {
    let app = Router::new()
        .nest("/permission", permission::generate_route())
        .nest("/slot", slot::generate_route())
        .nest("/sales-person", sales_person::generate_route())
        .nest("/booking", booking::generate_route())
        .with_state(rest_state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Could not bind server");
    axum::serve(listener, app)
        .await
        .expect("Could not start server");
}
