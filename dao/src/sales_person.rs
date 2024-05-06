use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::DaoError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SalesPersonEntity {
    pub id: Uuid,
    pub name: Arc<str>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub inactive: bool,
    pub version: Uuid,
}

#[automock]
#[async_trait]
pub trait SalesPersonDao {
    async fn all(&self) -> Result<Arc<[SalesPersonEntity]>, DaoError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SalesPersonEntity>, DaoError>;
    async fn create(&self, entity: &SalesPersonEntity, process: &str) -> Result<(), DaoError>;
    async fn update(&self, entity: &SalesPersonEntity, process: &str) -> Result<(), DaoError>;
}
