use crate::permission::Authentication;
use crate::ServiceError;
use async_trait::async_trait;
use mockall::automock;
use shifty_utils::DayOfWeek;
use std::fmt::Debug;
use std::sync::Arc;
use time::PrimitiveDateTime;

#[derive(Clone, Debug, PartialEq)]
pub struct BookingLog {
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeek,
    pub name: Arc<str>,
    pub time_from: time::Time,
    pub time_to: time::Time,
    pub created: PrimitiveDateTime,
    pub deleted: Option<PrimitiveDateTime>,
    pub created_by: Arc<str>,
    pub deleted_by: Option<Arc<str>>,
}

#[automock(type Context=(); type Transaction = dao::MockTransaction;)]
#[async_trait]
pub trait BookingLogService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn get_booking_logs_for_week(
        &self,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[BookingLog]>, ServiceError>;
}
