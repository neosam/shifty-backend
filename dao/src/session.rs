use std::sync::Arc;

use crate::DaoError;
use async_trait::async_trait;
use mockall::automock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionEntity {
    pub id: Arc<str>,
    pub user_id: Arc<str>,
    pub expires: i64,
    pub created: i64,
}

#[automock]
#[async_trait]

pub trait SessionDao {
    async fn create(&self, entity: &SessionEntity) -> Result<(), DaoError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<SessionEntity>, DaoError>;
    async fn delete(&self, id: &str) -> Result<(), DaoError>;
}
