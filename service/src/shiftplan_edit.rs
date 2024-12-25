use crate::permission::Authentication;
use crate::slot::Slot;
use crate::ServiceError;
use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use uuid::Uuid;

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait ShiftplanEditService {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction + std::fmt::Debug + Clone + Send + Sync + 'static;

    async fn modify_slot(
        &self,
        slot: &Slot,
        change_year: u32,
        change_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Slot, ServiceError>;
    async fn remove_slot(
        &self,
        slot: Uuid,
        change_year: u32,
        change_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
