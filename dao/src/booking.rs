use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::DaoError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookingEntity {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub slot_id: Uuid,
    pub calendar_week: i32,
    pub year: u32,
    pub created: PrimitiveDateTime,
    pub deleted: Option<PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait BookingDao {
    type Transaction: crate::Transaction;

    async fn all(&self, tx: Self::Transaction) -> Result<Arc<[BookingEntity]>, DaoError>;
    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<BookingEntity>, DaoError>;
    async fn find_by_slot_id_from(
        &self,
        slot_id: Uuid,
        year: u32,
        week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[BookingEntity]>, DaoError>;
    async fn find_by_booking_data(
        &self,
        sales_person_id: Uuid,
        slot_id: Uuid,
        calendar_week: i32,
        year: u32,
        tx: Self::Transaction,
    ) -> Result<Option<BookingEntity>, DaoError>;
    async fn find_by_week(
        &self,
        calendar_week: u8,
        year: u32,
        tx: Self::Transaction,
    ) -> Result<Arc<[BookingEntity]>, DaoError>;
    async fn create(
        &self,
        entity: &BookingEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
    async fn update(
        &self,
        entity: &BookingEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}
