use serde::{Deserialize, Serialize};
use std::{convert::Infallible, sync::Arc};
use uuid::Uuid;

use axum::{
    body::Body,
    extract::State,
    response::Response,
    routing::{get, post},
    Json, Router,
};

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

async fn root<HelloService: service::HelloService>(
    State(hello_service): State<Arc<HelloService>>,
) -> Response {
    error_handler(
        (async {
            let string = hello_service.hello().await?;
            Ok(RoString::from(string).into())
        })
        .await,
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(default)]
    pub id: Uuid,
    pub name: String,
}

async fn add_user(Json(user): Json<User>) -> Response {
    println!("Adding user: {:?}", user);
    Response::builder().status(200).body(Body::empty()).unwrap()
}

pub async fn start_server<HelloService>(hello_service: HelloService)
where
    HelloService: service::HelloService + Send + Sync + 'static,
{
    let app = Router::new()
        .route("/", get(root))
        .route("/user", post(add_user))
        .with_state(Arc::new(hello_service));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Could not bind server");
    axum::serve(listener, app)
        .await
        .expect("Could not start server");
}
