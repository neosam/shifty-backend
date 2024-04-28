use std::{convert::Infallible, sync::Arc};

mod hello;
mod permission;

use axum::{body::Body, response::Response, routing::get, Router};

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

fn error_handler(result: Result<Response, service::ServiceError>) -> Response {
    match result {
        Ok(response) => response,
        Err(service::ServiceError::Forbidden) => {
            Response::builder().status(403).body(Body::empty()).unwrap()
        }
        Err(service::ServiceError::DatabaseQueryError(e)) => Response::builder()
            .status(500)
            .body(Body::new(e.to_string()))
            .unwrap(),
    }
}

pub trait RestStateDef: Clone + Send + Sync + 'static {
    type HelloService: service::HelloService + Send + Sync + 'static;
    type PermissionService: service::PermissionService + Send + Sync + 'static;

    fn hello_service(&self) -> Arc<Self::HelloService>;
    fn permission_service(&self) -> Arc<Self::PermissionService>;
}

pub async fn start_server<RestState: RestStateDef>(rest_state: RestState) {
    let app = Router::new()
        .route("/", get(hello::hello::<RestState>))
        .nest("/permission", permission::generate_route())
        .with_state(rest_state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Could not bind server");
    axum::serve(listener, app)
        .await
        .expect("Could not start server");
}
