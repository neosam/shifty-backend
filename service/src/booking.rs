use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
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
            version: booking.version,
        })
    }
}

#[async_trait]
pub trait BookingService {
    type Context: Clone + PartialEq + Eq + Debug + Send + Sync;

    async fn get_all(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[Booking]>, ServiceError>;
    async fn get(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<Booking, ServiceError>;
    async fn create(
        &self,
        booking: &Booking,
        context: Authentication<Self::Context>,
    ) -> Result<Booking, ServiceError>;
    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError>;
}
