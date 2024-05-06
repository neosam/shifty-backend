use std::sync::Arc;

use axum::body::Body;
use axum::extract::Path;
use axum::routing::{delete, get, post, put};
use axum::{extract::State, response::Response};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use service::sales_person::SalesPerson;
use service::sales_person::SalesPersonService;
use uuid::Uuid;

use crate::{error_handler, RestError, RestStateDef};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SalesPersonTO {
    #[serde(default)]
    pub id: Uuid,
    pub name: Arc<str>,
    #[serde(default)]
    pub inactive: bool,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
impl From<&SalesPerson> for SalesPersonTO {
    fn from(sales_person: &SalesPerson) -> Self {
        Self {
            id: sales_person.id,
            name: sales_person.name.clone(),
            inactive: sales_person.inactive,
            deleted: sales_person.deleted,
            version: sales_person.version,
        }
    }
}
impl From<&SalesPersonTO> for SalesPerson {
    fn from(sales_person: &SalesPersonTO) -> Self {
        Self {
            id: sales_person.id,
            name: sales_person.name.clone(),
            inactive: sales_person.inactive,
            deleted: sales_person.deleted,
            version: sales_person.version,
        }
    }
}

pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_all_sales_persons::<RestState>))
        .route("/:id", get(get_sales_person::<RestState>))
        .route("/", post(create_sales_person::<RestState>))
        .route("/:id", put(update_sales_person::<RestState>))
        .route("/:id", delete(delete_sales_person::<RestState>))
}

pub async fn get_all_sales_persons<RestState: RestStateDef>(
    rest_state: State<RestState>,
) -> Response {
    error_handler(
        (async {
            let sales_persons: Arc<[SalesPersonTO]> = rest_state
                .sales_person_service()
                .get_all(())
                .await?
                .iter()
                .map(SalesPersonTO::from)
                .collect();
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&sales_persons).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn get_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            let sales_person = SalesPersonTO::from(
                &rest_state
                    .sales_person_service()
                    .get(sales_person_id, ())
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&sales_person).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn create_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Json(sales_person): Json<SalesPersonTO>,
) -> Response {
    error_handler(
        (async {
            let sales_person = SalesPersonTO::from(
                &rest_state
                    .sales_person_service()
                    .create(&(&sales_person).into(), ())
                    .await?,
            );
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&sales_person).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn update_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Path(sales_person_id): Path<Uuid>,
    Json(sales_person): Json<SalesPersonTO>,
) -> Response {
    error_handler(
        (async {
            if sales_person_id != sales_person.id {
                return Err(RestError::InconsistentId(sales_person_id, sales_person.id));
            }
            rest_state
                .sales_person_service()
                .update(&(&sales_person).into(), ())
                .await?;
            Ok(Response::builder()
                .status(200)
                .body(Body::new(serde_json::to_string(&sales_person).unwrap()))
                .unwrap())
        })
        .await,
    )
}

pub async fn delete_sales_person<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Path(sales_person_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .sales_person_service()
                .delete(sales_person_id, ())
                .await?;
            Ok(Response::builder().status(204).body(Body::empty()).unwrap())
        })
        .await,
    )
}
