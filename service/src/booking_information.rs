use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use uuid::Uuid;

use crate::booking::Booking;
use crate::permission::Authentication;
use crate::sales_person::SalesPerson;
use crate::slot::Slot;
use crate::ServiceError;

#[derive(Clone, Debug, PartialEq)]
pub struct BookingInformation {
    pub booking: Booking,
    pub slot: Arc<Slot>,
    pub sales_person: Arc<SalesPerson>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkingHoursPerSalesPerson {
    pub sales_person_id: Uuid,
    pub sales_person_name: Arc<str>,
    pub available_hours: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeeklySummary {
    pub year: u32,
    pub week: u8,
    pub overall_available_hours: f32,
    pub required_hours: f32,

    pub monday_available_hours: f32,
    pub tuesday_available_hours: f32,
    pub wednesday_available_hours: f32,
    pub thursday_available_hours: f32,
    pub friday_available_hours: f32,
    pub saturday_available_hours: f32,
    pub sunday_available_hours: f32,

    pub working_hours_per_sales_person: Arc<[WorkingHoursPerSalesPerson]>,
}

pub fn build_booking_information(
    slots: Arc<[Slot]>,
    booking: Arc<[Booking]>,
    sales_persons: Arc<[SalesPerson]>,
) -> Arc<[BookingInformation]> {
    let mut slot_map: HashMap<Uuid, Arc<Slot>> = HashMap::new();
    let mut sales_person_map: HashMap<Uuid, Arc<SalesPerson>> = HashMap::new();
    for slot in slots.iter() {
        slot_map.insert(slot.id, slot.clone().into());
    }
    for sales_person in sales_persons.iter() {
        sales_person_map.insert(sales_person.id, sales_person.clone().into());
    }
    let booking_informations = booking
        .iter()
        .filter_map(|booking| {
            Some(BookingInformation {
                booking: booking.clone(),
                slot: slot_map.get(&booking.slot_id)?.clone(),
                sales_person: sales_person_map.get(&booking.sales_person_id)?.clone(),
            })
        })
        .collect();

    booking_informations
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait BookingInformationService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn get_booking_conflicts_for_week(
        &self,
        years: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[BookingInformation]>, ServiceError>;

    async fn get_weekly_summary(
        &self,
        years: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[WeeklySummary]>, ServiceError>;

    async fn get_summery_for_week(
        &self,
        years: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<WeeklySummary, ServiceError>;
}
