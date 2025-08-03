use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use shifty_utils::DayOfWeek;
use uuid::Uuid;

use crate::DaoError;

#[derive(Debug, PartialEq, Eq)]
pub struct SlotEntity {
    pub id: Uuid,
    pub day_of_week: DayOfWeek,
    pub from: time::Time,
    pub to: time::Time,
    pub min_resources: u8,
    pub valid_from: time::Date,
    pub valid_to: Option<time::Date>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait SlotDao {
    type Transaction: crate::Transaction;

    async fn get_slots(&self, tx: Self::Transaction) -> Result<Arc<[SlotEntity]>, DaoError>;
    async fn get_slot(
        &self,
        id: &Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<SlotEntity>, DaoError>;
    async fn get_slots_for_week(
        &self,
        year: u32,
        week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[SlotEntity]>, DaoError>;
    async fn create_slot(
        &self,
        slot: &SlotEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
    //async fn delete_slot(&self, id: &Uuid, process: &str) -> Result<(), DaoError>;
    async fn update_slot(
        &self,
        slot: &SlotEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}
