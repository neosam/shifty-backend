use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::DaoError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShiftplanEntity {
    pub id: Uuid,
    pub name: Arc<str>,
    pub is_planning: bool,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait ShiftplanDao {
    type Transaction: crate::Transaction;

    async fn all(&self, tx: Self::Transaction) -> Result<Arc<[ShiftplanEntity]>, DaoError>;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<ShiftplanEntity>, DaoError>;

    async fn create(
        &self,
        entity: &ShiftplanEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn update(
        &self,
        entity: &ShiftplanEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}
