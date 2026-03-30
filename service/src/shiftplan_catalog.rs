use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shiftplan {
    pub id: Uuid,
    pub name: Arc<str>,
    pub is_planning: bool,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

impl From<&dao::shiftplan::ShiftplanEntity> for Shiftplan {
    fn from(entity: &dao::shiftplan::ShiftplanEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name.clone(),
            is_planning: entity.is_planning,
            deleted: entity.deleted,
            version: entity.version,
        }
    }
}

impl From<&Shiftplan> for dao::shiftplan::ShiftplanEntity {
    fn from(shiftplan: &Shiftplan) -> Self {
        Self {
            id: shiftplan.id,
            name: shiftplan.name.clone(),
            is_planning: shiftplan.is_planning,
            deleted: shiftplan.deleted,
            version: shiftplan.version,
        }
    }
}

#[automock(type Context=(); type Transaction = MockTransaction;)]
#[async_trait]
pub trait ShiftplanService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn get_all(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Shiftplan]>, ServiceError>;

    async fn get_by_id(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Shiftplan, ServiceError>;

    async fn create(
        &self,
        shiftplan: &Shiftplan,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Shiftplan, ServiceError>;

    async fn update(
        &self,
        shiftplan: &Shiftplan,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Shiftplan, ServiceError>;

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
