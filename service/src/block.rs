use std::sync::Arc;

use crate::permission::Authentication;
use crate::ServiceError;
use crate::{
    booking::Booking,
    sales_person::SalesPerson,
    slot::{DayOfWeek, Slot},
};
use async_trait::async_trait;
use mockall::automock;
use time::Time;
use uuid::Uuid;

/// A `Block` groups consecutive bookings that share the same sales person,
/// day of week, and contiguous time range. For example, if a sales person
/// booked 9:00–10:00, 10:00–11:00, and 11:00–12:00 on Monday, all those
/// bookings (and their corresponding slots) would appear in a single `Block`
/// from 9:00 to 12:00. This type is *not* stored in the database.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// The sales person to whom these consecutive bookings belong.
    pub sales_person: Arc<SalesPerson>,
    /// The day of the week these bookings fall on (e.g., Monday).
    pub day_of_week: DayOfWeek,
    /// The earliest start time among all contained slots/bookings.
    pub from: Time,
    /// The latest end time among all contained slots/bookings.
    pub to: Time,
    /// The bookings included in this block. Each booking connects the sales person,
    /// a slot, and a specific calendar week.
    pub bookings: Arc<[Booking]>,
    /// The corresponding slots for these bookings. Each slot defines from/to time,
    /// day of week, and other slot metadata.
    pub slots: Arc<[Slot]>,
}

/// A service trait for grouping consecutive bookings into `Block`s.
#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait BlockService {
    /// Same pattern used by other services for the `Context` type.
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    /// Transaction type from your DAO layer.
    type Transaction: dao::Transaction;

    /// Returns all `Block`s for a given sales person in the specified year and calendar week.
    /// Consecutive bookings on the same day are merged into one block if the adjacent slots
    /// line up perfectly (i.e., previous slot’s `to` == next slot’s `from`).
    async fn get_blocks_for_sales_person_week(
        &self,
        sales_person_id: Uuid,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Block]>, ServiceError>;
}
