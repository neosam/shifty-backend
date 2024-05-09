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

#[automock]
#[async_trait]
pub trait BookingDao {
    async fn all(&self) -> Result<Arc<[BookingEntity]>, DaoError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<BookingEntity>, DaoError>;
    async fn find_by_booking_data(
        &self,
        sales_person_id: Uuid,
        slot_id: Uuid,
        calendar_week: i32,
        year: u32,
    ) -> Result<Option<BookingEntity>, DaoError>;
    async fn create(&self, entity: &BookingEntity, process: &str) -> Result<(), DaoError>;
    async fn update(&self, entity: &BookingEntity, process: &str) -> Result<(), DaoError>;
}
