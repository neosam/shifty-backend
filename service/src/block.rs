use std::sync::Arc;

use crate::{booking::Booking, sales_person::SalesPerson};

/// A set of multiple bookings from one sales person.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub sales_person: Arc<SalesPerson>,
    pub bookings: Arc<[Booking]>,
}
