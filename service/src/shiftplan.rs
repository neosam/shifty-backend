use crate::slot::DayOfWeek;
use crate::{permission::Authentication, ServiceError};
use dao::Transaction;
use mockall::automock;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct ShiftplanWeek {
    pub year: u32,
    pub calendar_week: u8,
    pub days: Vec<ShiftplanDay>,
}

#[derive(Debug, Clone)]
pub struct ShiftplanDay {
    pub day_of_week: DayOfWeek,
    pub slots: Vec<ShiftplanSlot>,
}

#[derive(Debug, Clone)]
pub struct ShiftplanSlot {
    pub slot: crate::slot::Slot,
    pub bookings: Vec<ShiftplanBooking>,
}

#[derive(Debug, Clone)]
pub struct ShiftplanBooking {
    pub booking: crate::booking::Booking,
    pub sales_person: crate::sales_person::SalesPerson,
}

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait::async_trait]
pub trait ShiftplanService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: Transaction;

    async fn get_shiftplan_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ShiftplanWeek, ServiceError>;
}
