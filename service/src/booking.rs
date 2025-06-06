use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Booking {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub slot_id: Uuid,
    pub calendar_week: i32,
    pub year: u32,
    pub created: Option<PrimitiveDateTime>,
    pub deleted: Option<PrimitiveDateTime>,
    pub created_by: Option<Arc<str>>,
    pub deleted_by: Option<Arc<str>>,
    pub version: Uuid,
}

impl From<&dao::booking::BookingEntity> for Booking {
    fn from(booking: &dao::booking::BookingEntity) -> Self {
        Self {
            id: booking.id,
            sales_person_id: booking.sales_person_id,
            slot_id: booking.slot_id,
            calendar_week: booking.calendar_week,
            year: booking.year,
            created: Some(booking.created),
            deleted: booking.deleted,
            created_by: booking.created_by.clone(),
            deleted_by: booking.deleted_by.clone(),
            version: booking.version,
        }
    }
}

impl TryFrom<&Booking> for dao::booking::BookingEntity {
    type Error = ServiceError;
    fn try_from(booking: &Booking) -> Result<Self, Self::Error> {
        Ok(Self {
            id: booking.id,
            sales_person_id: booking.sales_person_id,
            slot_id: booking.slot_id,
            calendar_week: booking.calendar_week,
            year: booking.year,
            created: booking.created.ok_or_else(|| ServiceError::InternalError)?,
            deleted: booking.deleted,
            created_by: booking.created_by.clone(),
            deleted_by: booking.deleted_by.clone(),
            version: booking.version,
        })
    }
}

#[automock(type Context = (); type Transaction = dao::MockTransaction;)]
#[async_trait]
pub trait BookingService {
    type Context: Clone + PartialEq + Eq + Debug + Send + Sync;
    type Transaction: dao::Transaction;

    async fn get_all(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Booking]>, ServiceError>;
    async fn get(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Booking, ServiceError>;
    async fn get_for_week(
        &self,
        calendar_week: u8,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Booking]>, ServiceError>;
    async fn get_for_slot_id_since(
        &self,
        slot_id: Uuid,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Booking]>, ServiceError>;
    async fn create(
        &self,
        booking: &Booking,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Booking, ServiceError>;
    async fn copy_week(
        &self,
        from_calendar_week: u8,
        from_year: u32,
        to_calendar_week: u8,
        to_year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
